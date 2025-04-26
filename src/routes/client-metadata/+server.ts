import { json } from "@sveltejs/kit";
export const prerender = import.meta.env.MODE === 'native';

export function GET() {
  const client_uri = "https://tirekick.same.supply";

  return json({
    client_id: `${client_uri}/client-metadata`,
    application_type: "native",
    grant_types: [
      "authorization_code",
      "refresh_token"
    ],
    scope: "atproto transition:generic",
    response_types: [
      "code"
    ],
    redirect_uris: [
      "supply.same.tirekick:/callback/atproto",
      `${client_uri}/callback/atproto`
    ],
    token_endpoint_auth_method: "none",
    // token_endpoint_auth_signing_alg: 'ES256',
    dpop_bound_access_tokens: true,
    // (jwks: Jwks[] ) || (jwks_uri: url string)
    client_name: "Tire Kick",
    client_uri: `${client_uri}`,
    logo_uri: `${client_uri}/favicon.png`,
    // tos_uri: url string,
    // policy_uri: url string
  })
}
