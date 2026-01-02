use aws_cognito_srp::{SrpClient, User};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Error, Result};

pub struct HoAuth {
    pub jwt_token: String,
    pub id_token: String,
    pub access_token: String,
}

const HO_API_URI: &str = "https://api-myaccount.hydroottawa.com";
const COGNITO_ENDPOINT: &str = "https://cognito-idp.ca-central-1.amazonaws.com/";
const CLIENT_ID: &str = "7scfcis6ecucktmp4aqi1jk6cb";
const USER_POOL_ID: &str = "ca-central-1_VYnwOhMBK";

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct InitiateAuthRequest {
    auth_flow: String,
    client_id: String,
    auth_parameters: HashMap<String, String>,
    client_metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct InitiateAuthResponse {
    //challenge_name: String,
    challenge_parameters: ChallengeParameters,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct ChallengeParameters {
    salt: String,
    secret_block: String,
    srp_b: String,
    username: String,
    user_id_for_srp: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct RespondToAuthChallengeRequest {
    challenge_name: String,
    client_id: String,
    challenge_responses: HashMap<String, String>,
    client_metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthenticationResult {
    access_token: String,
    //expires_in: u32,
    id_token: String,
    //refresh_token: String,
    //token_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RespondToAuthChallengeResponse {
    authentication_result: AuthenticationResult,
}

impl HoAuth {
    pub async fn new<U, P>(username: U, password: P) -> Result<Self>
    where
        U: AsRef<str>,
        P: AsRef<str>,
    {
        let client = Client::new();

        let username = username.as_ref();
        let password = password.as_ref();

        // Step 1: Create SRP client and generate authentication parameters
        let user = User::new(USER_POOL_ID, username, password);
        let srp_client = SrpClient::new(user, CLIENT_ID, None); // None = no client secret

        let auth_params = srp_client.get_auth_parameters();

        // Step 2: Initiate auth with SRP_A
        let mut auth_parameters = HashMap::new();
        auth_parameters.insert("USERNAME".to_string(), auth_params.username.clone());
        auth_parameters.insert("SRP_A".to_string(), auth_params.a.clone());

        let initiate_request = InitiateAuthRequest {
            auth_flow: "USER_SRP_AUTH".to_string(),
            client_id: CLIENT_ID.to_string(),
            auth_parameters,
            client_metadata: HashMap::new(),
        };

        let initiate_response = client
            .post(COGNITO_ENDPOINT)
            .header("Content-Type", "application/x-amz-json-1.1")
            .header(
                "X-Amz-Target",
                "AWSCognitoIdentityProviderService.InitiateAuth",
            )
            .json(&initiate_request)
            .send()
            .await?
            .json::<InitiateAuthResponse>()
            .await?;

        // Step 3: Verify the challenge and generate password verifier
        let verification = srp_client.verify(
            &initiate_response.challenge_parameters.secret_block,
            &initiate_response.challenge_parameters.user_id_for_srp,
            &initiate_response.challenge_parameters.salt,
            &initiate_response.challenge_parameters.srp_b,
        )?;

        // Step 4: Build challenge response
        let mut challenge_responses = HashMap::new();
        challenge_responses.insert(
            "USERNAME".to_string(),
            initiate_response.challenge_parameters.username.clone(),
        );
        challenge_responses.insert(
            "PASSWORD_CLAIM_SECRET_BLOCK".to_string(),
            verification.password_claim_secret_block,
        );
        challenge_responses.insert("TIMESTAMP".to_string(), verification.timestamp);
        challenge_responses.insert(
            "PASSWORD_CLAIM_SIGNATURE".to_string(),
            verification.password_claim_signature,
        );

        let respond_request = RespondToAuthChallengeRequest {
            challenge_name: "PASSWORD_VERIFIER".to_string(),
            client_id: CLIENT_ID.to_string(),
            challenge_responses,
            client_metadata: HashMap::new(),
        };

        // Step 5: Respond to challenge and get tokens
        let auth_result = client
            .post(COGNITO_ENDPOINT)
            .header("Content-Type", "application/x-amz-json-1.1")
            .header(
                "X-Amz-Target",
                "AWSCognitoIdentityProviderService.RespondToAuthChallenge",
            )
            .json(&respond_request)
            .send()
            .await?
            .json::<RespondToAuthChallengeResponse>()
            .await?;

        // Step 6: Exchange Cognito tokens for Hydro Ottawa JWT
        let app_token_url = format!("{HO_API_URI}/app-token");
        let response = client
            .get(&app_token_url)
            .header("Accept", "application/json")
            .header("x-id", &auth_result.authentication_result.id_token)
            .header("x-access", &auth_result.authentication_result.access_token)
            .send()
            .await?;

        // Extract the custom JWT from the response header
        let jwt_token = response
            .headers()
            .get("x-amzn-remapped-authorization")
            .ok_or_else(|| Error::MissingHeader("x-amzn-remapped-authorization".to_string()))?
            .to_str()?
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                Error::InvalidTokenFormat("Token doesn't start with 'Bearer '".to_string())
            })?
            .to_string();

        Ok(Self {
            jwt_token,
            id_token: auth_result.authentication_result.id_token,
            access_token: auth_result.authentication_result.access_token,
        })
    }
}
