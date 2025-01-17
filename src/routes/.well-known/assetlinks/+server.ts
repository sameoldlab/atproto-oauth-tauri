import { json } from "@sveltejs/kit"

export function GET() {
  let CERT_FINGERPRINT = 'TODO!'
  return json([
    {
      "relation": [
        "delegate_permission/common.handle_all_urls"
      ],
      "target": {
        "namespace": "android_app",
        "package_name": "supply.same.tirekick",
        "sha256_cert_fingerprints": [
          CERT_FINGERPRINT
        ]
      }
    }
  ])
}
