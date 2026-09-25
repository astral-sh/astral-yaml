#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("gh-deployment"), @r##"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "name" [plain @1:1..1:5]: "Deployment" [plain @1:7..1:17]
      "run-name" [plain @2:1..2:9]: "${{ inputs.tag_name }} / ${{ inputs.environment }}${{ inputs.dry_run == true && ' (dry run)' || '' }}" [plain @2:11..2:112]
      "concurrency" [plain @4:1..4:12]: mapping [block]
        "group" [plain @5:3..5:8]: "${{ github.workflow }}-${{ github.ref_name }}" [plain @5:10..5:55]
        "cancel-in-progress" [plain @6:3..6:21]: "true" [plain @6:23..6:27]
      "permissions" [plain @8:1..8:12]: mapping [block]
        "attestations" [plain @9:3..9:15]: "write" [plain @9:17..9:22]
        "contents" [plain @10:3..10:11]: "write" [plain @10:13..10:18]
        "id-token" [plain @11:3..11:11]: "write" [plain @11:13..11:18]
      "on" [plain @13:1..13:3]: mapping [block]
        "workflow_dispatch" [plain @14:3..14:20]: mapping [block]
          "inputs" [plain @15:5..15:11]: mapping [block]
            "tag_name" [plain @16:7..16:15]: mapping [block]
              "required" [plain @17:9..17:17]: "true" [plain @17:19..17:23]
              "type" [plain @18:9..18:13]: "string" [plain @18:15..18:21]
              "description" [plain @19:9..19:20]: "The tag name for the release (e.g. v2.100.0, or v2.100.0-rc.1 for a pre-release)." [double-quoted @19:22..19:105]
            "environment" [plain @20:7..20:18]: mapping [block]
              "default" [plain @21:9..21:16]: "production" [plain @21:18..21:28]
              "type" [plain @22:9..22:13]: "environment" [plain @22:15..22:26]
              "description" [plain @23:9..23:20]: "The deployment environment." [double-quoted @23:22..23:51]
            "platforms" [plain @24:7..24:16]: mapping [block]
              "default" [plain @25:9..25:16]: "linux,macos,windows" [double-quoted @25:18..25:39]
              "type" [plain @26:9..26:13]: "string" [plain @26:15..26:21]
              "description" [plain @27:9..27:20]: "Comma-separated list of platforms to build." [double-quoted @27:22..27:67]
            "release" [plain @28:7..28:14]: mapping [block]
              "description" [plain @29:9..29:20]: "Whether to run the final release job. the dry_run flag still blocks final submissions." [double-quoted @29:22..29:110]
              "type" [plain @30:9..30:13]: "boolean" [plain @30:15..30:22]
              "default" [plain @31:9..31:16]: "true" [plain @31:18..31:22]
            "dry_run" [plain @32:7..32:14]: mapping [block]
              "description" [plain @33:9..33:20]: "Perform a dry run without publishing artifacts or creating a release" [double-quoted @33:22..33:92]
              "type" [plain @34:9..34:13]: "boolean" [plain @34:15..34:22]
              "default" [plain @35:9..35:16]: "true" [plain @35:18..35:22]
      "jobs" [plain @37:1..37:5]: mapping [block]
        "validate-tag-name" [plain @38:3..38:20]: mapping [block]
          "runs-on" [plain @39:5..39:12]: "ubuntu-latest" [plain @39:14..39:27]
          "steps" [plain @40:5..40:10]: sequence [block]
            - mapping [block]
              "name" [plain @41:9..41:13]: "Validate tag name format" [plain @41:15..41:39]
              "env" [plain @42:9..42:12]: mapping [block]
                "TAG_NAME" [plain @43:11..43:19]: "${{ inputs.tag_name }}" [plain @43:21..43:43]
              "run" [plain @44:9..44:12]: literal [@44:14..50:1]
                "# Build metadata (v1.2.3+001) is rejected: later steps detect a pre-release by its hyphen.\n"
                "if [[ ! \"$TAG_NAME\" =~ ^v[0-9]+\\.[0-9]+\\.[0-9]+(-[0-9A-Za-z-]+(\\.[0-9A-Za-z-]+)*)?$ ]]; then\n"
                "  echo \"Invalid tag name format. Must be in the form v1.2.3 or v1.2.3-rc.1\"\n"
                "  exit 1\n"
                "fi\n"
        "linux" [plain @50:3..50:8]: mapping [block]
          "needs" [plain @51:5..51:10]: "validate-tag-name" [plain @51:12..51:29]
          "runs-on" [plain @52:5..52:12]: "ubuntu-latest" [plain @52:14..52:27]
          "environment" [plain @53:5..53:16]: "${{ inputs.environment }}" [plain @53:18..53:43]
          "if" [plain @54:5..54:7]: "contains(inputs.platforms, 'linux')" [plain @54:9..54:44]
          "timeout-minutes" [plain @55:5..55:20]: "20" [plain @55:22..55:24]
          "steps" [plain @56:5..56:10]: sequence [block]
            - mapping [block]
              "name" [plain @57:9..57:13]: "Checkout" [plain @57:15..57:23]
              "uses" [plain @58:9..58:13]: "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1" [plain @58:15..58:72]
            - mapping [block]
              "name" [plain @59:9..59:13]: "Set up Go" [plain @59:15..59:24]
              "uses" [plain @60:9..60:13]: "actions/setup-go@b7ad1dad31e06c5925ef5d2fc7ad053ef454303e" [plain @60:15..60:72]
              "with" [plain @61:9..61:13]: mapping [block]
                "go-version-file" [plain @62:11..62:26]: "go.mod" [single-quoted @62:28..62:36]
            - mapping [block]
              "name" [plain @63:9..63:13]: "Install GoReleaser" [plain @63:15..63:33]
              "uses" [plain @64:9..64:13]: "goreleaser/goreleaser-action@f06c13b6b1a9625abc9e6e439d9c05a8f2190e94" [plain @64:15..64:84]
              "with" [plain @65:9..65:13]: mapping [block]
                "version" [plain @68:11..68:18]: "v2.13.1" [plain @68:20..68:27]
                "install-only" [plain @69:11..69:23]: "true" [plain @69:25..69:29]
            - mapping [block]
              "name" [plain @72:9..72:13]: "Create temporary tag" [plain @72:15..72:35]
              "env" [plain @73:9..73:12]: mapping [block]
                "TAG_NAME" [plain @74:11..74:19]: "${{ inputs.tag_name }}" [plain @74:21..74:43]
              "run" [plain @75:9..75:12]: "git tag \"$TAG_NAME\"" [plain @75:14..75:33]
            - mapping [block]
              "name" [plain @76:9..76:13]: "Build release binaries" [plain @76:15..76:37]
              "env" [plain @77:9..77:12]: mapping [block]
                "TAG_NAME" [plain @78:11..78:19]: "${{ inputs.tag_name }}" [plain @78:21..78:43]
              "run" [plain @79:9..79:12]: "script/release --local \"$TAG_NAME\" --platform linux" [plain @79:14..79:65]
            - mapping [block]
              "name" [plain @80:9..80:13]: "Generate web manual pages" [plain @80:15..80:40]
              "run" [plain @81:9..81:12]: literal [@81:14..84:1]
                "go run ./cmd/gen-docs --website --doc-path dist/manual\n"
                "tar -czvf dist/manual.tar.gz -C dist -- manual\n"
            - mapping [block]
              "uses" [plain @84:9..84:13]: "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a" [plain @84:15..84:79]
              "with" [plain @85:9..85:13]: mapping [block]
                "name" [plain @86:11..86:15]: "linux" [plain @86:17..86:22]
                "if-no-files-found" [plain @87:11..87:28]: "error" [plain @87:30..87:35]
                "retention-days" [plain @88:11..88:25]: "7" [plain @88:27..88:28]
                "path" [plain @89:11..89:15]: literal [@89:17..94:1]
                  "dist/*.tar.gz\n"
                  "dist/*.rpm\n"
                  "dist/*.deb\n"
        "macos" [plain @94:3..94:8]: mapping [block]
          "needs" [plain @95:5..95:10]: "validate-tag-name" [plain @95:12..95:29]
          "runs-on" [plain @96:5..96:12]: "macos-latest" [plain @96:14..96:26]
          "environment" [plain @97:5..97:16]: "${{ inputs.environment }}" [plain @97:18..97:43]
          "if" [plain @98:5..98:7]: "contains(inputs.platforms, 'macos')" [plain @98:9..98:44]
          "timeout-minutes" [plain @99:5..99:20]: "20" [plain @99:22..99:24]
          "steps" [plain @100:5..100:10]: sequence [block]
            - mapping [block]
              "name" [plain @101:9..101:13]: "Checkout" [plain @101:15..101:23]
              "uses" [plain @102:9..102:13]: "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1" [plain @102:15..102:72]
            - mapping [block]
              "name" [plain @103:9..103:13]: "Set up Go" [plain @103:15..103:24]
              "uses" [plain @104:9..104:13]: "actions/setup-go@b7ad1dad31e06c5925ef5d2fc7ad053ef454303e" [plain @104:15..104:72]
              "with" [plain @105:9..105:13]: mapping [block]
                "go-version-file" [plain @106:11..106:26]: "go.mod" [single-quoted @106:28..106:36]
            - mapping [block]
              "name" [plain @108:9..108:13]: "Install code signing certificate" [plain @108:15..108:47]
              "if" [plain @109:9..109:11]: "inputs.environment == 'production'" [plain @109:13..109:47]
              "shell" [plain @110:9..110:14]: "bash" [plain @110:16..110:20]
              "env" [plain @111:9..111:12]: mapping [block]
                "DEVELOPER_ID_CERT" [plain @112:11..112:28]: "${{ secrets.GATEWATCHER_DEVELOPER_ID_CERT }}" [plain @112:30..112:74]
                "DEVELOPER_ID_CERT_PASSWORD" [plain @113:11..113:37]: "${{ secrets.GATEWATCHER_DEVELOPER_ID_PASSWORD }}" [plain @113:39..113:87]
              "run" [plain @114:9..114:12]: literal [@114:14..128:1]
                "# create a keychain for the certificate\n"
                "PW=pwd.${{ github.run_number }}\n"
                "security create-keychain -p $PW \"$RUNNER_TEMP/build.keychain\"\n"
                "security set-keychain-settings -lut 21600 \"$RUNNER_TEMP/build.keychain\"\n"
                "security default-keychain -s \"$RUNNER_TEMP/build.keychain\"\n"
                "security unlock-keychain -p $PW \"$RUNNER_TEMP/build.keychain\"\n"
                "\n"
                "# import the certificate\n"
                "base64 -d <<< \"$DEVELOPER_ID_CERT\" > \"$RUNNER_TEMP/cert.p12\"\n"
                "security import \"$RUNNER_TEMP/cert.p12\" -k \"$RUNNER_TEMP/build.keychain\" -P \"$DEVELOPER_ID_CERT_PASSWORD\" -T /usr/bin/codesign\n"
                "security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k $PW \"$RUNNER_TEMP/build.keychain\"\n"
                "rm \"$RUNNER_TEMP/cert.p12\"\n"
            - mapping [block]
              "name" [plain @128:9..128:13]: "Add App Store Connect API key to keychain" [plain @128:15..128:56]
              "if" [plain @129:9..129:11]: "inputs.environment == 'production'" [plain @129:13..129:47]
              "uses" [plain @130:9..130:13]: "nodeselector/setup-apple-codesign@6b2173bdcb7dd4057e0396ccfa28ebfa4d90affa" [plain @130:15..130:89]
              "id" [plain @131:9..131:11]: "setup-apple-codesign" [plain @131:13..131:33]
              "with" [plain @132:9..132:13]: mapping [block]
                "asset-type" [plain @133:11..133:21]: "app-store-connect-api-key" [double-quoted @133:23..133:50]
                "app-store-connect-api-key-key-id" [plain @134:11..134:43]: "${{ secrets.GATEWATCHER_APP_STORE_CONNECT_API_KEY_ID }}" [plain @134:45..134:100]
                "app-store-connect-api-key-issuer-id" [plain @135:11..135:46]: "${{ secrets.GATEWATCHER_APP_STORE_CONNECT_API_ISSUER_ID }}" [plain @135:48..135:106]
                "app-store-connect-api-key-base64-private-key" [plain @136:11..136:55]: "${{ secrets.GATEWATCHER_APP_STORE_CONNECT_API_BASE64_PRIVATE_KEY }}" [plain @136:57..136:124]
            - mapping [block]
              "name" [plain @138:9..138:13]: "Configure notarization credentials" [plain @138:15..138:49]
              "if" [plain @139:9..139:11]: "inputs.environment == 'production'" [plain @139:13..139:47]
              "shell" [plain @140:9..140:14]: "bash" [plain @140:16..140:20]
              "run" [plain @141:9..141:12]: literal [@141:14..148:1]
                "xcrun notarytool store-credentials \"notarytool-password\" \\\n"
                "  --key \"${{ steps.setup-apple-codesign.outputs.app-store-connect-api-key-key-path }}\" \\\n"
                "  --key-id \"${{ steps.setup-apple-codesign.outputs.app-store-connect-api-key-key-id }}\" \\\n"
                "  --issuer \"${{ steps.setup-apple-codesign.outputs.app-store-connect-api-key-issuer-id }}\" \\\n"
                "  --keychain \"$RUNNER_TEMP/build.keychain\"\n"
            - mapping [block]
              "name" [plain @148:9..148:13]: "Install GoReleaser" [plain @148:15..148:33]
              "uses" [plain @149:9..149:13]: "goreleaser/goreleaser-action@f06c13b6b1a9625abc9e6e439d9c05a8f2190e94" [plain @149:15..149:84]
              "with" [plain @150:9..150:13]: mapping [block]
                "version" [plain @153:11..153:18]: "v2.13.1" [plain @153:20..153:27]
                "install-only" [plain @154:11..154:23]: "true" [plain @154:25..154:29]
            - mapping [block]
              "name" [plain @157:9..157:13]: "Create temporary tag" [plain @157:15..157:35]
              "env" [plain @158:9..158:12]: mapping [block]
                "TAG_NAME" [plain @159:11..159:19]: "${{ inputs.tag_name }}" [plain @159:21..159:43]
              "run" [plain @160:9..160:12]: "git tag \"$TAG_NAME\"" [plain @160:14..160:33]
            - mapping [block]
              "name" [plain @161:9..161:13]: "Build release binaries" [plain @161:15..161:37]
              "env" [plain @162:9..162:12]: mapping [block]
                "TAG_NAME" [plain @163:11..163:19]: "${{ inputs.tag_name }}" [plain @163:21..163:43]
                "KEYCHAIN" [plain @164:11..164:19]: "${{ runner.temp }}/build.keychain" [plain @164:21..164:54]
                "DEVELOPER_ID_CERT_IDENTIFIER" [plain @165:11..165:39]: "${{ vars.MAC_APP_SIGNING_IDENTITY }}" [plain @165:41..165:77]
                "DO_SIGN_ARTIFACTS" [plain @166:11..166:28]: "${{ inputs.environment == 'production' }}" [plain @166:30..166:71]
              "run" [plain @167:9..167:12]: "script/release --local \"$TAG_NAME\" --platform macos" [plain @167:14..167:65]
            - mapping [block]
              "name" [plain @168:9..168:13]: "Notarize macOS archives" [plain @168:15..168:38]
              "if" [plain @169:9..169:11]: "inputs.environment == 'production'" [plain @169:13..169:47]
              "env" [plain @170:9..170:12]: mapping [block]
                "DEVELOPER_ID_CERT_IDENTIFIER" [plain @171:11..171:39]: "${{ vars.MAC_APP_SIGNING_IDENTITY }}" [plain @171:41..171:77]
                "KEYCHAIN" [plain @172:11..172:19]: "${{ runner.temp }}/build.keychain" [plain @172:21..172:54]
                "DO_SIGN_ARTIFACTS" [plain @173:11..173:28]: "${{ inputs.environment == 'production' }}" [plain @173:30..173:71]
              "run" [plain @174:9..174:12]: literal [@174:14..177:1]
                "shopt -s failglob\n"
                "script/sign dist/gh_*_macOS_*.zip\n"
            - mapping [block]
              "name" [plain @177:9..177:13]: "Build universal macOS pkg installer" [plain @177:15..177:50]
              "if" [plain @178:9..178:11]: "inputs.environment != 'production'" [plain @178:13..178:47]
              "env" [plain @179:9..179:12]: mapping [block]
                "TAG_NAME" [plain @180:11..180:19]: "${{ inputs.tag_name }}" [plain @180:21..180:43]
              "run" [plain @181:9..181:12]: "script/pkgmacos \"$TAG_NAME\"" [plain @181:14..181:41]
            - mapping [block]
              "name" [plain @182:9..182:13]: "Build & notarize universal macOS pkg installer" [plain @182:15..182:61]
              "if" [plain @183:9..183:11]: "inputs.environment == 'production'" [plain @183:13..183:47]
              "env" [plain @184:9..184:12]: mapping [block]
                "TAG_NAME" [plain @185:11..185:19]: "${{ inputs.tag_name }}" [plain @185:21..185:43]
                "APPLE_DEVELOPER_INSTALLER_ID" [plain @186:11..186:39]: "${{ vars.APPLE_DEVELOPER_INSTALLER_ID }}" [plain @186:41..186:81]
              "run" [plain @187:9..187:12]: literal [@187:14..190:1]
                "shopt -s failglob\n"
                "script/pkgmacos \"$TAG_NAME\"\n"
            - mapping [block]
              "uses" [plain @190:9..190:13]: "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a" [plain @190:15..190:79]
              "with" [plain @191:9..191:13]: mapping [block]
                "name" [plain @192:11..192:15]: "macos" [plain @192:17..192:22]
                "if-no-files-found" [plain @193:11..193:28]: "error" [plain @193:30..193:35]
                "retention-days" [plain @194:11..194:25]: "7" [plain @194:27..194:28]
                "path" [plain @195:11..195:15]: literal [@195:17..200:1]
                  "dist/*.tar.gz\n"
                  "dist/*.zip\n"
                  "dist/*.pkg\n"
        "windows" [plain @200:3..200:10]: mapping [block]
          "needs" [plain @201:5..201:10]: "validate-tag-name" [plain @201:12..201:29]
          "runs-on" [plain @202:5..202:12]: "windows-2022" [plain @202:14..202:26]
          "environment" [plain @203:5..203:16]: "${{ inputs.environment }}" [plain @203:18..203:43]
          "if" [plain @204:5..204:7]: "contains(inputs.platforms, 'windows')" [plain @204:9..204:46]
          "timeout-minutes" [plain @205:5..205:20]: "20" [plain @205:22..205:24]
          "steps" [plain @206:5..206:10]: sequence [block]
            - mapping [block]
              "name" [plain @207:9..207:13]: "Checkout" [plain @207:15..207:23]
              "uses" [plain @208:9..208:13]: "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1" [plain @208:15..208:72]
            - mapping [block]
              "name" [plain @209:9..209:13]: "Set up Go" [plain @209:15..209:24]
              "uses" [plain @210:9..210:13]: "actions/setup-go@b7ad1dad31e06c5925ef5d2fc7ad053ef454303e" [plain @210:15..210:72]
              "with" [plain @211:9..211:13]: mapping [block]
                "go-version-file" [plain @212:11..212:26]: "go.mod" [single-quoted @212:28..212:36]
            - mapping [block]
              "name" [plain @213:9..213:13]: "Install GoReleaser" [plain @213:15..213:33]
              "uses" [plain @214:9..214:13]: "goreleaser/goreleaser-action@f06c13b6b1a9625abc9e6e439d9c05a8f2190e94" [plain @214:15..214:84]
              "with" [plain @215:9..215:13]: mapping [block]
                "version" [plain @218:11..218:18]: "v2.13.1" [plain @218:20..218:27]
                "install-only" [plain @219:11..219:23]: "true" [plain @219:25..219:29]
            - mapping [block]
              "name" [plain @220:9..220:13]: "Install Azure Code Signing Client" [plain @220:15..220:48]
              "shell" [plain @221:9..221:14]: "pwsh" [plain @221:16..221:20]
              "env" [plain @222:9..222:12]: mapping [block]
                "ACS_DIR" [plain @223:11..223:18]: "${{ runner.temp }}\\acs" [plain @223:20..223:42]
                "ACS_ZIP" [plain @224:11..224:18]: "${{ runner.temp }}\\acs.zip" [plain @224:20..224:46]
                "CORRELATION_ID" [plain @225:11..225:25]: "${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}" [plain @225:27..225:110]
                "METADATA_PATH" [plain @226:11..226:24]: "${{ runner.temp }}\\acs\\metadata.json" [plain @226:26..226:62]
              "run" [plain @227:9..227:12]: literal [@227:14..240:1]
                "# Download Azure Code Signing client containing the DLL needed for signtool in script/sign\n"
                "Invoke-WebRequest -Uri https://www.nuget.org/api/v2/package/Microsoft.Trusted.Signing.Client/1.0.95 -OutFile $Env:ACS_ZIP -Verbose\n"
                "Expand-Archive $Env:ACS_ZIP -Destination $Env:ACS_DIR -Force -Verbose\n"
                "\n"
                "# Generate metadata file for signtool, used in signing box .exe and .msi\n"
                "@{\n"
                "  CertificateProfileName = \"GitHubInc\"\n"
                "  CodeSigningAccountName = \"GitHubInc\"\n"
                "  CorrelationId = $Env:CORRELATION_ID\n"
                "  Endpoint =  \"https://wus3.codesigning.azure.net/\"\n"
                "} | ConvertTo-Json | Out-File -FilePath $Env:METADATA_PATH\n"
            - mapping [block]
              "name" [plain @242:9..242:13]: "Create temporary tag" [plain @242:15..242:35]
              "shell" [plain @243:9..243:14]: "bash" [plain @243:16..243:20]
              "env" [plain @244:9..244:12]: mapping [block]
                "TAG_NAME" [plain @245:11..245:19]: "${{ inputs.tag_name }}" [plain @245:21..245:43]
              "run" [plain @246:9..246:12]: "git tag \"$TAG_NAME\"" [plain @246:14..246:33]
            - mapping [block]
              "name" [plain @247:9..247:13]: "Authenticate to Azure for code signing" [plain @247:15..247:53]
              "if" [plain @248:9..248:11]: "inputs.environment == 'production'" [plain @248:13..248:47]
              "uses" [plain @249:9..249:13]: "azure/login@a641126d1b8aa4d1fa005f4f92df94a3a4c4c906" [plain @249:15..249:67]
              "with" [plain @250:9..250:13]: mapping [block]
                "client-id" [plain @251:11..251:20]: "${{ secrets.SPN_GITHUB_CLI_SIGNING_CLIENT_ID }}" [plain @251:22..251:69]
                "tenant-id" [plain @252:11..252:20]: "${{ secrets.SPN_GITHUB_CLI_SIGNING_TENANT_ID }}" [plain @252:22..252:69]
                "allow-no-subscriptions" [plain @253:11..253:33]: "true" [plain @253:35..253:39]
            - mapping [block]
              "name" [plain @256:9..256:13]: "Build release binaries" [plain @256:15..256:37]
              "shell" [plain @257:9..257:14]: "bash" [plain @257:16..257:20]
              "env" [plain @258:9..258:12]: mapping [block]
                "AZURE_CLIENT_ID" [plain @259:11..259:26]: "${{ secrets.SPN_GITHUB_CLI_SIGNING_CLIENT_ID }}" [plain @259:28..259:75]
                "AZURE_TENANT_ID" [plain @260:11..260:26]: "${{ secrets.SPN_GITHUB_CLI_SIGNING_TENANT_ID }}" [plain @260:28..260:75]
                "DLIB_PATH" [plain @261:11..261:20]: "${{ runner.temp }}\\acs\\bin\\x64\\Azure.CodeSigning.Dlib.dll" [plain @261:22..261:79]
                "METADATA_PATH" [plain @262:11..262:24]: "${{ runner.temp }}\\acs\\metadata.json" [plain @262:26..262:62]
                "TAG_NAME" [plain @263:11..263:19]: "${{ inputs.tag_name }}" [plain @263:21..263:43]
                "DO_SIGN_ARTIFACTS" [plain @264:11..264:28]: "${{ inputs.environment == 'production' }}" [plain @264:30..264:71]
              "run" [plain @265:9..265:12]: "script/release --local \"$TAG_NAME\" --platform windows" [plain @265:14..265:67]
            - mapping [block]
              "name" [plain @266:9..266:13]: "Set up MSBuild" [plain @266:15..266:29]
              "id" [plain @267:9..267:11]: "setupmsbuild" [plain @267:13..267:25]
              "uses" [plain @268:9..268:13]: "microsoft/setup-msbuild@30375c66a4eea26614e0d39710365f22f8b0af57" [plain @268:15..268:79]
            - mapping [block]
              "name" [plain @269:9..269:13]: "Build MSI" [plain @269:15..269:24]
              "shell" [plain @270:9..270:14]: "bash" [plain @270:16..270:20]
              "env" [plain @271:9..271:12]: mapping [block]
                "MSBUILD_PATH" [plain @272:11..272:23]: "${{ steps.setupmsbuild.outputs.msbuildPath }}" [plain @272:25..272:70]
              "run" [plain @273:9..273:12]: literal [@273:14..297:1]
                "for ZIP_FILE in dist/gh_*_windows_*.zip; do\n"
                "  MSI_NAME=\"$(basename \"$ZIP_FILE\" \".zip\")\"\n"
                "  MSI_VERSION=\"$(cut -d_ -f2 <<<\"$MSI_NAME\" | cut -d- -f1)\"\n"
                "  case \"$MSI_NAME\" in\n"
                "  *_386 )\n"
                "    source_dir=\"$PWD/dist/windows_windows_386_sse2\"\n"
                "    platform=\"x86\"\n"
                "    ;;\n"
                "  *_amd64 )\n"
                "    source_dir=\"$PWD/dist/windows_windows_amd64_v1\"\n"
                "    platform=\"x64\"\n"
                "    ;;\n"
                "  *_arm64 )\n"
                "    source_dir=\"$PWD/dist/windows_windows_arm64_v8.0\"\n"
                "    platform=\"arm64\"\n"
                "    ;;\n"
                "  * )\n"
                "    printf \"unsupported architecture: %s\\n\" \"$MSI_NAME\" >&2\n"
                "    exit 1\n"
                "    ;;\n"
                "  esac\n"
                "  \"${MSBUILD_PATH}\\MSBuild.exe\" ./build/windows/gh.wixproj -p:SourceDir=\"$source_dir\" -p:OutputPath=\"$PWD/dist\" -p:OutputName=\"$MSI_NAME\" -p:ProductVersion=\"${MSI_VERSION#v}\" -p:Platform=\"$platform\"\n"
                "done\n"
            - mapping [block]
              "name" [plain @297:9..297:13]: "Sign .msi release binaries" [plain @297:15..297:41]
              "if" [plain @298:9..298:11]: "inputs.environment == 'production'" [plain @298:13..298:47]
              "shell" [plain @299:9..299:14]: "pwsh" [plain @299:16..299:20]
              "env" [plain @300:9..300:12]: mapping [block]
                "AZURE_CLIENT_ID" [plain @301:11..301:26]: "${{ secrets.SPN_GITHUB_CLI_SIGNING_CLIENT_ID }}" [plain @301:28..301:75]
                "AZURE_TENANT_ID" [plain @302:11..302:26]: "${{ secrets.SPN_GITHUB_CLI_SIGNING_TENANT_ID }}" [plain @302:28..302:75]
                "DLIB_PATH" [plain @303:11..303:20]: "${{ runner.temp }}\\acs\\bin\\x64\\Azure.CodeSigning.Dlib.dll" [plain @303:22..303:79]
                "METADATA_PATH" [plain @304:11..304:24]: "${{ runner.temp }}\\acs\\metadata.json" [plain @304:26..304:62]
                "DO_SIGN_ARTIFACTS" [plain @305:11..305:28]: "${{ inputs.environment == 'production' }}" [plain @305:30..305:71]
              "run" [plain @306:9..306:12]: literal [@306:14..310:1]
                "Get-ChildItem -Path .\\dist -Filter *.msi | ForEach-Object {\n"
                "  .\\script\\sign.ps1 $_.FullName\n"
                "}\n"
            - mapping [block]
              "uses" [plain @310:9..310:13]: "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a" [plain @310:15..310:79]
              "with" [plain @311:9..311:13]: mapping [block]
                "name" [plain @312:11..312:15]: "windows" [plain @312:17..312:24]
                "if-no-files-found" [plain @313:11..313:28]: "error" [plain @313:30..313:35]
                "retention-days" [plain @314:11..314:25]: "7" [plain @314:27..314:28]
                "path" [plain @315:11..315:15]: literal [@315:17..319:1]
                  "dist/*.zip\n"
                  "dist/*.msi\n"
        "release" [plain @319:3..319:10]: mapping [block]
          "runs-on" [plain @320:5..320:12]: "ubuntu-latest" [plain @320:14..320:27]
          "needs" [plain @321:5..321:10]: sequence [flow]
            - "linux" [plain @321:13..321:18]
            - "macos" [plain @321:20..321:25]
            - "windows" [plain @321:27..321:34]
          "environment" [plain @322:5..322:16]: "${{ inputs.environment }}" [plain @322:18..322:43]
          "if" [plain @323:5..323:7]: "inputs.release" [plain @323:9..323:23]
          "steps" [plain @324:5..324:10]: sequence [block]
            - mapping [block]
              "name" [plain @325:9..325:13]: "Checkout cli/cli" [plain @325:15..325:31]
              "uses" [plain @326:9..326:13]: "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1" [plain @326:15..326:72]
            - mapping [block]
              "name" [plain @327:9..327:13]: "Merge built artifacts" [plain @327:15..327:36]
              "uses" [plain @328:9..328:13]: "actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c" [plain @328:15..328:81]
            - mapping [block]
              "name" [plain @329:9..329:13]: "Generate site deploy token" [plain @329:15..329:41]
              "id" [plain @330:9..330:11]: "site-deploy-token" [plain @330:13..330:30]
              "if" [plain @331:9..331:11]: "inputs.environment == 'production'" [plain @331:13..331:47]
              "uses" [plain @332:9..332:13]: "actions/create-github-app-token@bcd2ba49218906704ab6c1aa796996da409d3eb1" [plain @332:15..332:87]
              "with" [plain @333:9..333:13]: mapping [block]
                "client-id" [plain @334:11..334:20]: "${{ secrets.SITE_DEPLOY_APP_CLIENT_ID }}" [plain @334:22..334:62]
                "private-key" [plain @335:11..335:22]: "${{ secrets.SITE_DEPLOY_APP_PRIVATE_KEY }}" [plain @335:24..335:66]
                "owner" [plain @336:11..336:16]: "github" [plain @336:18..336:24]
                "repositories" [plain @337:11..337:23]: "cli.github.com" [plain @337:25..337:39]
            - mapping [block]
              "name" [plain @338:9..338:13]: "Checkout documentation site" [plain @338:15..338:42]
              "if" [plain @339:9..339:11]: "${{ inputs.environment == 'production' }}" [plain @339:13..339:54]
              "uses" [plain @340:9..340:13]: "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1" [plain @340:15..340:72]
              "with" [plain @341:9..341:13]: mapping [block]
                "repository" [plain @342:11..342:21]: "github/cli.github.com" [plain @342:23..342:44]
                "path" [plain @343:11..343:15]: "site" [plain @343:17..343:21]
                "fetch-depth" [plain @344:11..344:22]: "0" [plain @344:24..344:25]
                "token" [plain @345:11..345:16]: "${{ steps.site-deploy-token.outputs.token}}" [plain @345:18..345:61]
            - mapping [block]
              "name" [plain @346:9..346:13]: "Update site man pages" [plain @346:15..346:36]
              "if" [plain @347:9..347:11]: "${{ inputs.environment == 'production' }}" [plain @347:13..347:54]
              "env" [plain @348:9..348:12]: mapping [block]
                "GIT_COMMITTER_NAME" [plain @349:11..349:29]: "cli automation" [plain @349:31..349:45]
                "GIT_AUTHOR_NAME" [plain @350:11..350:26]: "cli automation" [plain @350:28..350:42]
                "GIT_COMMITTER_EMAIL" [plain @351:11..351:30]: "noreply@github.com" [plain @351:32..351:50]
                "GIT_AUTHOR_EMAIL" [plain @352:11..352:27]: "noreply@github.com" [plain @352:29..352:47]
                "TAG_NAME" [plain @353:11..353:19]: "${{ inputs.tag_name }}" [plain @353:21..353:43]
              "run" [plain @354:9..354:12]: literal [@354:14..362:1]
                "git -C site rm 'manual/gh*.md' 2>/dev/null || true\n"
                "tar -xzvf linux/manual.tar.gz -C site\n"
                "git -C site add 'manual/gh*.md'\n"
                "sed -i.bak -E \"s/(assign version = )\\\".+\\\"/\\1\\\"${TAG_NAME#v}\\\"/\" site/index.html\n"
                "rm -f site/index.html.bak\n"
                "git -C site add index.html\n"
                "git -C site diff --quiet --cached || git -C site commit -m \"gh ${TAG_NAME#v}\"\n"
            - mapping [block]
              "name" [plain @362:9..362:13]: "Prepare release assets" [plain @362:15..362:37]
              "env" [plain @363:9..363:12]: mapping [block]
                "TAG_NAME" [plain @364:11..364:19]: "${{ inputs.tag_name }}" [plain @364:21..364:43]
              "run" [plain @365:9..365:12]: literal [@365:14..370:1]
                "shopt -s failglob\n"
                "rm -rf dist\n"
                "mkdir dist\n"
                "mv -v {linux,macos,windows}/gh_* dist/\n"
            - mapping [block]
              "name" [plain @370:9..370:13]: "Install packaging dependencies" [plain @370:15..370:45]
              "run" [plain @371:9..371:12]: "sudo apt-get install -y rpm reprepro" [plain @371:14..371:50]
            - mapping [block]
              "name" [plain @372:9..372:13]: "Set up GPG" [plain @372:15..372:25]
              "if" [plain @373:9..373:11]: "inputs.environment == 'production'" [plain @373:13..373:47]
              "env" [plain @374:9..374:12]: mapping [block]
                "GPG_PUBKEY_2026" [plain @375:11..375:26]: "${{ secrets.GPG_PUBKEY_2026 }}" [plain @375:28..375:58]
                "GPG_KEY_2026" [plain @376:11..376:23]: "${{ secrets.GPG_KEY_2026 }}" [plain @376:25..376:52]
                "GPG_PASSPHRASE_2026" [plain @377:11..377:30]: "${{ secrets.GPG_PASSPHRASE_2026 }}" [plain @377:32..377:66]
                "GPG_KEYGRIP_2026" [plain @378:11..378:27]: "${{ secrets.GPG_KEYGRIP_2026 }}" [plain @378:29..378:60]
              "run" [plain @379:9..379:12]: literal [@379:14..385:1]
                "base64 -d <<<\"$GPG_PUBKEY_2026\" | gpg --import --no-tty --batch --yes\n"
                "base64 -d <<<\"$GPG_KEY_2026\" | gpg --import --no-tty --batch --yes\n"
                "echo \"allow-preset-passphrase\" > ~/.gnupg/gpg-agent.conf\n"
                "gpg-connect-agent RELOADAGENT /bye\n"
                "base64 -d <<<\"$GPG_PASSPHRASE_2026\" | /usr/lib/gnupg2/gpg-preset-passphrase --preset \"$GPG_KEYGRIP_2026\"\n"
            - mapping [block]
              "name" [plain @385:9..385:13]: "Sign RPMs" [plain @385:15..385:24]
              "if" [plain @386:9..386:11]: "inputs.environment == 'production'" [plain @386:13..386:47]
              "run" [plain @387:9..387:12]: literal [@387:14..390:1]
                "cp script/rpmmacros ~/.rpmmacros\n"
                "rpmsign --addsign dist/*.rpm\n"
            - mapping [block]
              "name" [plain @390:9..390:13]: "Attest release artifacts" [plain @390:15..390:39]
              "if" [plain @391:9..391:11]: "inputs.environment == 'production' && !inputs.dry_run" [plain @391:13..391:66]
              "uses" [plain @392:9..392:13]: "actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6" [plain @392:15..392:70]
              "with" [plain @393:9..393:13]: mapping [block]
                "subject-path" [plain @394:11..394:23]: "dist/gh_*" [double-quoted @394:25..394:36]
                "create-storage-record" [plain @395:11..395:32]: "false" [plain @395:34..395:39]
            - mapping [block]
              "name" [plain @396:9..396:13]: "Run createrepo" [plain @396:15..396:29]
              "if" [plain @397:9..397:11]: "${{ inputs.environment == 'production' }}" [plain @397:13..397:54]
              "run" [plain @398:9..398:12]: literal [@398:14..406:1]
                "mkdir -p site/packages/rpm\n"
                "cp dist/*.rpm site/packages/rpm/\n"
                "./script/createrepo.sh\n"
                "cp -r dist/repodata site/packages/rpm/\n"
                "pushd site/packages/rpm\n"
                "gpg --yes --detach-sign --armor --default-key 7F38BBB59D064DBCB3D84D725612B36462313325 repodata/repomd.xml\n"
                "popd\n"
            - mapping [block]
              "name" [plain @406:9..406:13]: "Run reprepro" [plain @406:15..406:27]
              "if" [plain @407:9..407:11]: "${{ inputs.environment == 'production' }}" [plain @407:13..407:54]
              "env" [plain @408:9..408:12]: mapping [block]
                "RELEASES" [plain @412:11..412:19]: "cosmic eoan disco groovy focal stable oldstable testing sid unstable buster bullseye stretch jessie bionic trusty precise xenial hirsute impish kali-rolling" [double-quoted @412:21..412:179]
              "run" [plain @413:9..413:12]: literal [@413:14..423:1]
                "mkdir -p upload\n"
                "for release in $RELEASES; do\n"
                "  for file in dist/*.deb; do\n"
                "    reprepro --confdir=\"+b/script\" includedeb \"$release\" \"$file\"\n"
                "  done\n"
                "done\n"
                "cp -a dists/ pool/ upload/\n"
                "mkdir -p site/packages\n"
                "cp -a upload/* site/packages/\n"
            - mapping [block]
              "name" [plain @423:9..423:13]: "Create the release" [plain @423:15..423:33]
              "env" [plain @424:9..424:12]: mapping [block]
                "DO_PUBLISH" [plain @426:11..426:21]: "${{ inputs.environment == 'production' && !inputs.dry_run }}" [plain @426:23..426:83]
                "TAG_NAME" [plain @427:11..427:19]: "${{ inputs.tag_name }}" [plain @427:21..427:43]
                "GH_TOKEN" [plain @428:11..428:19]: "${{ secrets.GITHUB_TOKEN }}" [plain @428:21..428:48]
              "run" [plain @429:9..429:12]: literal [@429:14..447:1]
                "shopt -s failglob\n"
                "pushd dist\n"
                "shasum -a 256 gh_* > checksums.txt\n"
                "mv checksums.txt gh_${TAG_NAME#v}_checksums.txt\n"
                "popd\n"
                "release_args=(\n"
                "  \"$TAG_NAME\"\n"
                "  --title \"GitHub CLI ${TAG_NAME#v}\"\n"
                "  --target \"$GITHUB_SHA\"\n"
                "  --generate-notes\n"
                ")\n"
                "if [[ $TAG_NAME == *-* ]]; then\n"
                "  release_args+=( --prerelease )\n"
                "fi\n"
                "guard=\"echo\"\n"
                "[ \"$DO_PUBLISH\" = \"false\" ] || guard=\"\"\n"
                "script/label-assets dist/gh_* | xargs $guard gh release create \"${release_args[@]}\" --\n"
            - mapping [block]
              "name" [plain @447:9..447:13]: "Publish site" [plain @447:15..447:27]
              "if" [plain @448:9..448:11]: "${{ inputs.environment == 'production' }}" [plain @448:13..448:54]
              "env" [plain @449:9..449:12]: mapping [block]
                "DO_PUBLISH" [plain @450:11..450:21]: "${{ inputs.environment == 'production' && !contains(inputs.tag_name, '-') && !inputs.dry_run }}" [plain @450:23..450:118]
                "TAG_NAME" [plain @451:11..451:19]: "${{ inputs.tag_name }}" [plain @451:21..451:43]
                "GIT_COMMITTER_NAME" [plain @452:11..452:29]: "cli automation" [plain @452:31..452:45]
                "GIT_AUTHOR_NAME" [plain @453:11..453:26]: "cli automation" [plain @453:28..453:42]
                "GIT_COMMITTER_EMAIL" [plain @454:11..454:30]: "noreply@github.com" [plain @454:32..454:50]
                "GIT_AUTHOR_EMAIL" [plain @455:11..455:27]: "noreply@github.com" [plain @455:29..455:47]
              "working-directory" [plain @456:9..456:26]: "./site" [plain @456:28..456:34]
              "run" [plain @457:9..457:12]: literal [@457:14..466:1]
                "git add packages\n"
                "git commit -m \"Add rpm and deb packages for $TAG_NAME\"\n"
                "if [ \"$DO_PUBLISH\" = \"true\" ]; then\n"
                "  git push\n"
                "else\n"
                "  git log --oneline @{upstream}..\n"
                "  git diff --name-status @{upstream}..\n"
                "fi\n"
    "##);
}
