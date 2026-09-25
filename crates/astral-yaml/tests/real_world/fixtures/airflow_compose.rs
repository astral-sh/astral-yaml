#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("airflow-compose"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "x-airflow-common" [plain @47:1..47:17]: mapping [block] &airflow-common
        "image" [plain @52:3..52:8]: "${AIRFLOW_IMAGE_NAME:-apache/airflow:|version|}" [plain @52:10..52:57]
        "env_file" [plain @54:3..54:11]: sequence [block]
          - "${ENV_FILE_PATH:-.env}" [plain @55:7..55:29]
        "environment" [plain @56:3..56:14]: mapping [block] &airflow-common-env
          "AIRFLOW__CORE__EXECUTOR" [plain @58:5..58:28]: "CeleryExecutor" [plain @58:30..58:44]
          "AIRFLOW__CORE__AUTH_MANAGER" [plain @59:5..59:32]: "airflow.providers.fab.auth_manager.fab_auth_manager.FabAuthManager" [plain @59:34..59:100]
          "AIRFLOW__DATABASE__SQL_ALCHEMY_CONN" [plain @60:5..60:40]: "postgresql+psycopg://airflow:airflow@postgres/airflow" [plain @60:42..60:95]
          "AIRFLOW__CELERY__RESULT_BACKEND" [plain @61:5..61:36]: "db+postgresql+psycopg://airflow:airflow@postgres/airflow" [plain @61:38..61:94]
          "AIRFLOW__CELERY__BROKER_URL" [plain @62:5..62:32]: "redis://:@redis:6379/0" [plain @62:34..62:56]
          "AIRFLOW__CORE__FERNET_KEY" [plain @63:5..63:30]: "${FERNET_KEY}" [plain @63:32..63:45]
          "AIRFLOW__CORE__DAGS_ARE_PAUSED_AT_CREATION" [plain @64:5..64:47]: "true" [single-quoted @64:49..64:55]
          "AIRFLOW__CORE__LOAD_EXAMPLES" [plain @65:5..65:33]: "true" [single-quoted @65:35..65:41]
          "AIRFLOW__CORE__EXECUTION_API_SERVER_URL" [plain @66:5..66:44]: "http://airflow-apiserver:8080/execution/" [single-quoted @66:46..66:88]
          "AIRFLOW__API_AUTH__JWT_SECRET" [plain @67:5..67:34]: "${AIRFLOW__API_AUTH__JWT_SECRET:-airflow_jwt_secret}" [plain @67:36..67:88]
          "AIRFLOW__API_AUTH__JWT_ISSUER" [plain @68:5..68:34]: "${AIRFLOW__API_AUTH__JWT_ISSUER:-airflow}" [plain @68:36..68:77]
          "AIRFLOW__SCHEDULER__ENABLE_HEALTH_CHECK" [plain @73:5..73:44]: "true" [single-quoted @73:46..73:52]
          "_PIP_ADDITIONAL_REQUIREMENTS" [plain @76:5..76:33]: "${_PIP_ADDITIONAL_REQUIREMENTS:-}" [plain @76:35..76:68]
          "AIRFLOW_CONFIG" [plain @78:5..78:19]: "/opt/airflow/config/airflow.cfg" [single-quoted @78:21..78:54]
        "volumes" [plain @79:3..79:10]: sequence [block]
          - "${AIRFLOW_PROJ_DIR:-.}/dags:/opt/airflow/dags" [plain @80:7..80:52]
          - "${AIRFLOW_PROJ_DIR:-.}/logs:/opt/airflow/logs" [plain @81:7..81:52]
          - "${AIRFLOW_PROJ_DIR:-.}/config:/opt/airflow/config" [plain @82:7..82:56]
          - "${AIRFLOW_PROJ_DIR:-.}/plugins:/opt/airflow/plugins" [plain @83:7..83:58]
        "user" [plain @84:3..84:7]: "${AIRFLOW_UID:-50000}:0" [double-quoted @84:9..84:34]
        "depends_on" [plain @85:3..85:13]: mapping [block] &airflow-common-depends-on
          "redis" [plain @87:5..87:10]: mapping [block]
            "condition" [plain @88:7..88:16]: "service_healthy" [plain @88:18..88:33]
          "postgres" [plain @89:5..89:13]: mapping [block]
            "condition" [plain @90:7..90:16]: "service_healthy" [plain @90:18..90:33]
      "services" [plain @92:1..92:9]: mapping [block]
        "postgres" [plain @93:3..93:11]: mapping [block]
          "image" [plain @94:5..94:10]: "postgres:16" [plain @94:12..94:23]
          "environment" [plain @95:5..95:16]: mapping [block]
            "POSTGRES_USER" [plain @96:7..96:20]: "airflow" [plain @96:22..96:29]
            "POSTGRES_PASSWORD" [plain @97:7..97:24]: "airflow" [plain @97:26..97:33]
            "POSTGRES_DB" [plain @98:7..98:18]: "airflow" [plain @98:20..98:27]
          "volumes" [plain @99:5..99:12]: sequence [block]
            - "postgres-db-volume:/var/lib/postgresql/data" [plain @100:9..100:52]
          "healthcheck" [plain @101:5..101:16]: mapping [block]
            "test" [plain @102:7..102:11]: sequence [flow]
              - "CMD" [double-quoted @102:14..102:19]
              - "pg_isready" [double-quoted @102:21..102:33]
              - "-U" [double-quoted @102:35..102:39]
              - "airflow" [double-quoted @102:41..102:50]
            "interval" [plain @103:7..103:15]: "10s" [plain @103:17..103:20]
            "retries" [plain @104:7..104:14]: "5" [plain @104:16..104:17]
            "start_period" [plain @105:7..105:19]: "5s" [plain @105:21..105:23]
          "restart" [plain @106:5..106:12]: "always" [plain @106:14..106:20]
        "redis" [plain @108:3..108:8]: mapping [block]
          "image" [plain @111:5..111:10]: "redis:7.2-bookworm" [plain @111:12..111:30]
          "expose" [plain @112:5..112:11]: sequence [block]
            - "6379" [plain @113:9..113:13]
          "healthcheck" [plain @114:5..114:16]: mapping [block]
            "test" [plain @115:7..115:11]: sequence [flow]
              - "CMD" [double-quoted @115:14..115:19]
              - "redis-cli" [double-quoted @115:21..115:32]
              - "ping" [double-quoted @115:34..115:40]
            "interval" [plain @116:7..116:15]: "10s" [plain @116:17..116:20]
            "timeout" [plain @117:7..117:14]: "30s" [plain @117:16..117:19]
            "retries" [plain @118:7..118:14]: "50" [plain @118:16..118:18]
            "start_period" [plain @119:7..119:19]: "30s" [plain @119:21..119:24]
          "restart" [plain @120:5..120:12]: "always" [plain @120:14..120:20]
        "airflow-apiserver" [plain @122:3..122:20]: mapping [block]
          "<<" [plain @123:5..123:7]: *airflow-common [@123:9..123:24]
          "command" [plain @124:5..124:12]: "api-server" [plain @124:14..124:24]
          "ports" [plain @125:5..125:10]: sequence [block]
            - "8080:8080" [double-quoted @126:9..126:20]
          "healthcheck" [plain @127:5..127:16]: mapping [block]
            "test" [plain @128:7..128:11]: sequence [flow]
              - "CMD" [double-quoted @128:14..128:19]
              - "curl" [double-quoted @128:21..128:27]
              - "--fail" [double-quoted @128:29..128:37]
              - "http://localhost:8080/api/v2/monitor/health" [double-quoted @128:39..128:84]
            "interval" [plain @129:7..129:15]: "30s" [plain @129:17..129:20]
            "timeout" [plain @130:7..130:14]: "10s" [plain @130:16..130:19]
            "retries" [plain @131:7..131:14]: "5" [plain @131:16..131:17]
            "start_period" [plain @132:7..132:19]: "30s" [plain @132:21..132:24]
          "restart" [plain @133:5..133:12]: "always" [plain @133:14..133:20]
          "depends_on" [plain @134:5..134:15]: mapping [block]
            "<<" [plain @135:7..135:9]: *airflow-common-depends-on [@135:11..135:37]
            "airflow-init" [plain @136:7..136:19]: mapping [block]
              "condition" [plain @137:9..137:18]: "service_completed_successfully" [plain @137:20..137:50]
        "airflow-scheduler" [plain @139:3..139:20]: mapping [block]
          "<<" [plain @140:5..140:7]: *airflow-common [@140:9..140:24]
          "command" [plain @141:5..141:12]: "scheduler" [plain @141:14..141:23]
          "healthcheck" [plain @142:5..142:16]: mapping [block]
            "test" [plain @143:7..143:11]: sequence [flow]
              - "CMD-SHELL" [double-quoted @143:14..143:25]
              - "airflow jobs check --job-type SchedulerJob --hostname \"$${HOSTNAME}\"" [single-quoted @143:27..143:97]
            "interval" [plain @144:7..144:15]: "30s" [plain @144:17..144:20]
            "timeout" [plain @145:7..145:14]: "10s" [plain @145:16..145:19]
            "retries" [plain @146:7..146:14]: "5" [plain @146:16..146:17]
            "start_period" [plain @147:7..147:19]: "30s" [plain @147:21..147:24]
          "restart" [plain @148:5..148:12]: "always" [plain @148:14..148:20]
          "depends_on" [plain @149:5..149:15]: mapping [block]
            "<<" [plain @150:7..150:9]: *airflow-common-depends-on [@150:11..150:37]
            "airflow-init" [plain @151:7..151:19]: mapping [block]
              "condition" [plain @152:9..152:18]: "service_completed_successfully" [plain @152:20..152:50]
        "airflow-dag-processor" [plain @154:3..154:24]: mapping [block]
          "<<" [plain @155:5..155:7]: *airflow-common [@155:9..155:24]
          "command" [plain @156:5..156:12]: "dag-processor" [plain @156:14..156:27]
          "healthcheck" [plain @157:5..157:16]: mapping [block]
            "test" [plain @158:7..158:11]: sequence [flow]
              - "CMD-SHELL" [double-quoted @158:14..158:25]
              - "airflow jobs check --job-type DagProcessorJob --hostname \"$${HOSTNAME}\"" [single-quoted @158:27..158:100]
            "interval" [plain @159:7..159:15]: "30s" [plain @159:17..159:20]
            "timeout" [plain @160:7..160:14]: "10s" [plain @160:16..160:19]
            "retries" [plain @161:7..161:14]: "5" [plain @161:16..161:17]
            "start_period" [plain @162:7..162:19]: "30s" [plain @162:21..162:24]
          "restart" [plain @163:5..163:12]: "always" [plain @163:14..163:20]
          "depends_on" [plain @164:5..164:15]: mapping [block]
            "<<" [plain @165:7..165:9]: *airflow-common-depends-on [@165:11..165:37]
            "airflow-init" [plain @166:7..166:19]: mapping [block]
              "condition" [plain @167:9..167:18]: "service_completed_successfully" [plain @167:20..167:50]
        "airflow-worker" [plain @169:3..169:17]: mapping [block]
          "<<" [plain @170:5..170:7]: *airflow-common [@170:9..170:24]
          "command" [plain @171:5..171:12]: "celery worker" [plain @171:14..171:27]
          "healthcheck" [plain @172:5..172:16]: mapping [block]
            "test" [plain @174:7..174:11]: sequence [flow]
              - "CMD-SHELL" [double-quoted @174:14..174:25]
              - "celery --app airflow.providers.celery.executors.celery_executor.app inspect ping -d \"celery@$${HOSTNAME}\" || celery --app airflow.executors.celery_executor.app inspect ping -d \"celery@$${HOSTNAME}\"" [single-quoted @174:27..174:226]
            "interval" [plain @175:7..175:15]: "30s" [plain @175:17..175:20]
            "timeout" [plain @176:7..176:14]: "10s" [plain @176:16..176:19]
            "retries" [plain @177:7..177:14]: "5" [plain @177:16..177:17]
            "start_period" [plain @178:7..178:19]: "30s" [plain @178:21..178:24]
          "environment" [plain @179:5..179:16]: mapping [block]
            "<<" [plain @180:7..180:9]: *airflow-common-env [@180:11..180:30]
            "DUMB_INIT_SETSID" [plain @183:7..183:23]: "0" [double-quoted @183:25..183:28]
          "restart" [plain @184:5..184:12]: "always" [plain @184:14..184:20]
          "depends_on" [plain @185:5..185:15]: mapping [block]
            "<<" [plain @186:7..186:9]: *airflow-common-depends-on [@186:11..186:37]
            "airflow-apiserver" [plain @187:7..187:24]: mapping [block]
              "condition" [plain @188:9..188:18]: "service_healthy" [plain @188:20..188:35]
            "airflow-init" [plain @189:7..189:19]: mapping [block]
              "condition" [plain @190:9..190:18]: "service_completed_successfully" [plain @190:20..190:50]
        "airflow-triggerer" [plain @192:3..192:20]: mapping [block]
          "<<" [plain @193:5..193:7]: *airflow-common [@193:9..193:24]
          "command" [plain @194:5..194:12]: "triggerer" [plain @194:14..194:23]
          "healthcheck" [plain @195:5..195:16]: mapping [block]
            "test" [plain @196:7..196:11]: sequence [flow]
              - "CMD-SHELL" [double-quoted @196:14..196:25]
              - "airflow jobs check --job-type TriggererJob --hostname \"$${HOSTNAME}\"" [single-quoted @196:27..196:97]
            "interval" [plain @197:7..197:15]: "30s" [plain @197:17..197:20]
            "timeout" [plain @198:7..198:14]: "10s" [plain @198:16..198:19]
            "retries" [plain @199:7..199:14]: "5" [plain @199:16..199:17]
            "start_period" [plain @200:7..200:19]: "30s" [plain @200:21..200:24]
          "restart" [plain @201:5..201:12]: "always" [plain @201:14..201:20]
          "depends_on" [plain @202:5..202:15]: mapping [block]
            "<<" [plain @203:7..203:9]: *airflow-common-depends-on [@203:11..203:37]
            "airflow-init" [plain @204:7..204:19]: mapping [block]
              "condition" [plain @205:9..205:18]: "service_completed_successfully" [plain @205:20..205:50]
        "airflow-init" [plain @207:3..207:15]: mapping [block]
          "<<" [plain @208:5..208:7]: *airflow-common [@208:9..208:24]
          "entrypoint" [plain @209:5..209:15]: "/bin/bash" [plain @209:17..209:26]
          "command" [plain @211:5..211:12]: sequence [block]
            - "-c" [plain @212:9..212:11]
            - literal [@213:9..289:1]
              "if [[ -z \"${AIRFLOW_UID}\" ]]; then\n"
              "  echo\n"
              "  echo -e \"\\033[1;33mWARNING!!!: AIRFLOW_UID not set!\\e[0m\"\n"
              "  echo \"If you are on Linux, you SHOULD follow the instructions below to set \"\n"
              "  echo \"AIRFLOW_UID environment variable, otherwise files will be owned by root.\"\n"
              "  echo \"For other operating systems you can get rid of the warning with manually created .env file:\"\n"
              "  echo \"    See: https://airflow.apache.org/docs/apache-airflow/stable/howto/docker-compose/index.html#setting-the-right-airflow-user\"\n"
              "  echo\n"
              "  export AIRFLOW_UID=$$(id -u)\n"
              "fi\n"
              "one_meg=1048576\n"
              "mem_available=$$(($$(getconf _PHYS_PAGES) * $$(getconf PAGE_SIZE) / one_meg))\n"
              "cpus_available=$$(grep -cE 'cpu[0-9]+' /proc/stat)\n"
              "disk_available=$$(df / | tail -1 | awk '{print $$4}')\n"
              "warning_resources=\"false\"\n"
              "if (( mem_available < 4000 )) ; then\n"
              "  echo\n"
              "  echo -e \"\\033[1;33mWARNING!!!: Not enough memory available for Docker.\\e[0m\"\n"
              "  echo \"At least 4GB of memory required. You have $$(numfmt --to iec $$((mem_available * one_meg)))\"\n"
              "  echo\n"
              "  warning_resources=\"true\"\n"
              "fi\n"
              "if (( cpus_available < 2 )); then\n"
              "  echo\n"
              "  echo -e \"\\033[1;33mWARNING!!!: Not enough CPUS available for Docker.\\e[0m\"\n"
              "  echo \"At least 2 CPUs recommended. You have $${cpus_available}\"\n"
              "  echo\n"
              "  warning_resources=\"true\"\n"
              "fi\n"
              "if (( disk_available < one_meg * 10 )); then\n"
              "  echo\n"
              "  echo -e \"\\033[1;33mWARNING!!!: Not enough Disk space available for Docker.\\e[0m\"\n"
              "  echo \"At least 10 GBs recommended. You have $$(numfmt --to iec $$((disk_available * 1024 )))\"\n"
              "  echo\n"
              "  warning_resources=\"true\"\n"
              "fi\n"
              "if [[ $${warning_resources} == \"true\" ]]; then\n"
              "  echo\n"
              "  echo -e \"\\033[1;33mWARNING!!!: You have not enough resources to run Airflow (see above)!\\e[0m\"\n"
              "  echo \"Please follow the instructions to increase amount of resources available:\"\n"
              "  echo \"   https://airflow.apache.org/docs/apache-airflow/stable/howto/docker-compose/index.html#before-you-begin\"\n"
              "  echo\n"
              "fi\n"
              "echo\n"
              "echo \"Creating missing opt dirs if missing:\"\n"
              "echo\n"
              "mkdir -v -p /opt/airflow/{logs,dags,plugins,config}\n"
              "echo\n"
              "echo \"Airflow version:\"\n"
              "/entrypoint airflow version\n"
              "echo\n"
              "echo \"Files in shared volumes:\"\n"
              "echo\n"
              "ls -la /opt/airflow/{logs,dags,plugins,config}\n"
              "echo\n"
              "echo \"Running airflow config list to create default config file if missing.\"\n"
              "echo\n"
              "/entrypoint airflow config list >/dev/null\n"
              "echo\n"
              "echo \"Files in shared volumes:\"\n"
              "echo\n"
              "ls -la /opt/airflow/{logs,dags,plugins,config}\n"
              "echo\n"
              "echo \"Change ownership of files in /opt/airflow to ${AIRFLOW_UID:-50000}:0\"\n"
              "echo\n"
              "chown -R \"${AIRFLOW_UID:-50000}:0\" /opt/airflow/\n"
              "echo\n"
              "echo \"Change ownership of files in shared volumes to ${AIRFLOW_UID:-50000}:0\"\n"
              "echo\n"
              "chown -v -R \"${AIRFLOW_UID:-50000}:0\" /opt/airflow/{logs,dags,plugins,config}\n"
              "echo\n"
              "echo \"Files in shared volumes:\"\n"
              "echo\n"
              "ls -la /opt/airflow/{logs,dags,plugins,config}\n"
          "environment" [plain @290:5..290:16]: mapping [block]
            "<<" [plain @291:7..291:9]: *airflow-common-env [@291:11..291:30]
            "_AIRFLOW_DB_MIGRATE" [plain @292:7..292:26]: "true" [single-quoted @292:28..292:34]
            "_AIRFLOW_WWW_USER_CREATE" [plain @293:7..293:31]: "true" [single-quoted @293:33..293:39]
            "_AIRFLOW_WWW_USER_USERNAME" [plain @294:7..294:33]: "${_AIRFLOW_WWW_USER_USERNAME:-airflow}" [plain @294:35..294:73]
            "_AIRFLOW_WWW_USER_PASSWORD" [plain @295:7..295:33]: "${_AIRFLOW_WWW_USER_PASSWORD:-airflow}" [plain @295:35..295:73]
            "_PIP_ADDITIONAL_REQUIREMENTS" [plain @296:7..296:35]: "" [single-quoted @296:37..296:39]
          "user" [plain @297:5..297:9]: "0:0" [double-quoted @297:11..297:16]
        "airflow-cli" [plain @299:3..299:14]: mapping [block]
          "<<" [plain @300:5..300:7]: *airflow-common [@300:9..300:24]
          "profiles" [plain @301:5..301:13]: sequence [block]
            - "debug" [plain @302:9..302:14]
          "environment" [plain @303:5..303:16]: mapping [block]
            "<<" [plain @304:7..304:9]: *airflow-common-env [@304:11..304:30]
            "CONNECTION_CHECK_MAX_COUNT" [plain @305:7..305:33]: "0" [double-quoted @305:35..305:38]
          "command" [plain @307:5..307:12]: sequence [block]
            - "bash" [plain @308:9..308:13]
            - "-c" [plain @309:9..309:11]
            - "airflow" [plain @310:9..310:16]
          "depends_on" [plain @311:5..311:15]: mapping [block]
            "<<" [plain @312:7..312:9]: *airflow-common-depends-on [@312:11..312:37]
        "flower" [plain @317:3..317:9]: mapping [block]
          "<<" [plain @318:5..318:7]: *airflow-common [@318:9..318:24]
          "command" [plain @319:5..319:12]: "celery flower" [plain @319:14..319:27]
          "profiles" [plain @320:5..320:13]: sequence [block]
            - "flower" [plain @321:9..321:15]
          "ports" [plain @322:5..322:10]: sequence [block]
            - "5555:5555" [double-quoted @323:9..323:20]
          "healthcheck" [plain @324:5..324:16]: mapping [block]
            "test" [plain @325:7..325:11]: sequence [flow]
              - "CMD" [double-quoted @325:14..325:19]
              - "curl" [double-quoted @325:21..325:27]
              - "--fail" [double-quoted @325:29..325:37]
              - "http://localhost:5555/" [double-quoted @325:39..325:63]
            "interval" [plain @326:7..326:15]: "30s" [plain @326:17..326:20]
            "timeout" [plain @327:7..327:14]: "10s" [plain @327:16..327:19]
            "retries" [plain @328:7..328:14]: "5" [plain @328:16..328:17]
            "start_period" [plain @329:7..329:19]: "30s" [plain @329:21..329:24]
          "restart" [plain @330:5..330:12]: "always" [plain @330:14..330:20]
          "depends_on" [plain @331:5..331:15]: mapping [block]
            "<<" [plain @332:7..332:9]: *airflow-common-depends-on [@332:11..332:37]
            "airflow-init" [plain @333:7..333:19]: mapping [block]
              "condition" [plain @334:9..334:18]: "service_completed_successfully" [plain @334:20..334:50]
      "volumes" [plain @336:1..336:8]: mapping [block]
        "postgres-db-volume" [plain @337:3..337:21]: "" [plain @337:22..337:22]
    "#);
}
