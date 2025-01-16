import { json } from "@sveltejs/kit";

export function GET() {
  const client_uri = "https://tirekick.same.supply";

  return json({
    client_id: `${client_uri}/client-metadata.json`,
    application_type: "native",
    grant_types: [
      "authorization_code",
      "refresh_token"
    ],
    scope: "atproto",
    response_types: [
      "code"
    ],
    redirect_uris: [
      "supply.same.tirekick:/callback/atproto",
      `${client_uri}/callback/atproto`
    ],
    dpop_bound_access_tokens: true,
    client_uri: `${client_uri}`,
    client_name: "Tire Kick"
  })
}
