import { json } from "@sveltejs/kit";

export function GET() {

  return json(
    {
      "applinks": {
        "details": [
          {
            "appIDs": ["$DEVELOPMENT_TEAM_ID.$APP_BUNDLE_ID"],
            "components": [
              {
                "/": "/open/*",
                "comment": "Matches any URL whose path starts with /open/"
              }
            ]
          }
        ]
      }
    }
  )
}
