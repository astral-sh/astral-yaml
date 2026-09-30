#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-api-servicecontrol-v1-servicecontrol-05af2a06"), @r###"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "servicecontrol.googleapis.com" [plain @3:7..3:36]
      "title" [plain @4:1..4:6]: "Service Control API" [plain @4:8..4:27]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.api.servicecontrol.v1.QuotaController" [plain @7:9..7:53]
        - mapping [block]
          "name" [plain @8:3..8:7]: "google.api.servicecontrol.v1.ServiceController" [plain @8:9..8:55]
      "documentation" [plain @10:1..10:14]: mapping [block]
        "summary" [plain @11:3..11:10]: literal [@11:12..14:1]
          "Provides admission control and telemetry reporting for services integrated\n"
          "with Service Infrastructure."
        "overview" [plain @14:3..14:11]: literal [@14:13..154:1]
          "Google Service Control provides control plane functionality to managed\n"
          "services, such as logging, monitoring, and status checks. This page\n"
          "provides an overview of what it does and how it works.\n"
          "\n"
          "## Why use Service Control?\n"
          "\n"
          "When you develop a cloud service, you typically start with the business\n"
          "requirements and the architecture design, then proceed with API definition\n"
          "and implementation. Before you put your service into production, you\n"
          "need to deal with many control plane issues:\n"
          "\n"
          "* How to control access to your service.\n"
          "* How to send logging and monitoring data to both consumers and producers.\n"
          "* How to create and manage dashboards to visualize this data.\n"
          "* How to automatically scale the control plane components with your\n"
          "service.\n"
          "\n"
          "Service Control is a mature and feature-rich control plane provider\n"
          "that addresses these needs with high efficiency, high scalability,\n"
          "and high availability. It provides a simple public API that can be\n"
          "accessed from anywhere using JSON REST and gRPC clients, so when you move\n"
          "your service from on-premise to a cloud provider, or from one cloud\n"
          "provider to another, you don't need to change the control plane provider.\n"
          "\n"
          "Services built using Google Cloud Endpoints already take advantage of\n"
          "Service Control. Cloud Endpoints sends logging and monitoring data\n"
          "through Google Service Control for every request arriving at its\n"
          "proxy. If you need to report any additional logging and monitoring data\n"
          "for your Cloud Endpoints service, you can call the Service Control API\n"
          "directly from your service.\n"
          "\n"
          "The Service Control API definition is open sourced and available on\n"
          "[GitHub](https://github.com/googleapis/googleapis/tree/master/google/api/servicecontrol). By\n"
          "changing the DNS name, you can easily use alternative implementations\n"
          "of the Service Control API.\n"
          "\n"
          "## Architecture\n"
          "\n"
          "Google Service Control works with a set of *managed services* and their\n"
          "*operations* (activities), *checks* whether an operation is allowed to\n"
          "proceed, and *reports* completed operations. Behind the scenes, it\n"
          "leverages other\n"
          "Google Cloud services, such as\n"
          "[Google Service\n"
          "Management](https://cloud.google.com/service-infrastructure/docs/service-management/getting-started), [Stackdriver\n"
          "Logging](https://cloud.google.com/products/operations), and [Stackdriver\n"
          "Monitoring](https://cloud.google.com/monitoring), while hiding their\n"
          "complexity from service producers. It enables service\n"
          "producers to send telemetry data to their consumers. It uses caching,\n"
          "batching, aggregation, and retries to deliver higher performance and\n"
          "availability than the individual backend systems it encapsulates.\n"
          "\n"
          "<figure id=\"fig-arch\" class=\"center\">\n"
          "<div style=\"width: 70%;margin: auto\">\n"
          "<img src=\"/service-infrastructure/docs/service-control/images/arch.svg\"\n"
          "alt=\"The overall architecture of a service that uses Google Service\n"
          "Control.\"> </div> <figcaption><b>Figure 1</b>: Using Google Service\n"
          "Control.</figcaption> </figure>\n"
          "\n"
          "The Service Control API provides two methods:\n"
          "\n"
          "*\n"
          "[`services.check`](https://cloud.google.com/service-infrastructure/docs/service-control/reference/rest/v1/services/check),\n"
          "used for:\n"
          "* Ensuring valid consumer status\n"
          "* Validating API keys\n"
          "*\n"
          "[`services.report`](https://cloud.google.com/service-infrastructure/docs/service-control/reference/rest/v1/services/report),\n"
          "used for:\n"
          "* Sending logs to Stackdriver Logging\n"
          "* Sending metrics to Stackdriver Monitoring\n"
          "\n"
          "We'll look at these in more detail in the rest of this overview.\n"
          "\n"
          "## Managed services\n"
          "\n"
          "A [managed\n"
          "service](https://cloud.google.com/service-infrastructure/docs/service-control/reference/rest/v1/services) is\n"
          "a network service managed by\n"
          "[Google Service\n"
          "Management](https://cloud.google.com/service-infrastructure/docs/service-management/getting-started). Each\n"
          "managed service has a unique name, such as `example.googleapis.com`,\n"
          "which must be a valid fully-qualified DNS name, as per RFC 1035.\n"
          "\n"
          "For example:\n"
          "\n"
          "* Google Cloud Pub/Sub (`pubsub.googleapis.com`)\n"
          "* Google Cloud Vision (`vision.googleapis.com`)\n"
          "* Google Cloud Bigtable (`bigtable.googleapis.com`)\n"
          "* Google Cloud Datastore (`datastore.googleapis.com`)\n"
          "\n"
          "Google Service Management manages the lifecycle of each service's\n"
          "configuration, which is used to customize Google Service Control's\n"
          "behavior. Service configurations are also used by Google Cloud Console for\n"
          "displaying APIs and their settings, enabling/disabling APIs, and more.\n"
          "\n"
          "## Operations\n"
          "\n"
          "Google Service Control uses the generic concept of an *operation*\n"
          "to represent the activities of a managed service, such as API calls and\n"
          "resource usage. Each operation is associated with a managed service and a\n"
          "specific service consumer, and has a set of properties that describe the\n"
          "operation, such as the API method name and resource usage amount. For more\n"
          "information, see the\n"
          "[Operation\n"
          "definition](https://cloud.google.com/service-infrastructure/docs/service-control/reference/rest/v1/Operation). ##\n"
          "Check\n"
          "\n"
          "The\n"
          "[`services.check`](https://cloud.google.com/service-infrastructure/docs/service-control/reference/rest/v1/services/check) method\n"
          "determines whether an operation should be allowed to proceed for a\n"
          "managed service.\n"
          "\n"
          "For example:\n"
          "\n"
          "* Check if the consumer is still active.\n"
          "* Check if the consumer has enabled the service.\n"
          "* Check if the API key is still valid.\n"
          "\n"
          "By performing multiple checks within a single method call, it provides\n"
          "better performance, higher reliability, and reduced development cost to\n"
          "service producers compared to checking with multiple backend systems.\n"
          "\n"
          "## Report\n"
          "\n"
          "The\n"
          "[`services.report`](https://cloud.google.com/service-infrastructure/docs/service-control/reference/rest/v1/services/report) method\n"
          "reports completed operations for a managed service to backend\n"
          "systems, such as logging and monitoring. The reported data can be seen in\n"
          "Google API Console and Google Cloud Console, and retrieved with\n"
          "appropriate APIs, such as the Stackdriver Logging and Stackdriver\n"
          "Monitoring APIs.\n"
          "\n"
          "## Next steps\n"
          "\n"
          "* Read our [Getting Started\n"
          "guide](/service-infrastructure/docs/service-control/getting-started)   to\n"
          "find out how to set up and use the Google Service Control API."
      "backend" [plain @154:1..154:8]: mapping [block]
        "rules" [plain @155:3..155:8]: sequence [block]
          - mapping [block]
            "selector" [plain @156:5..156:13]: "google.api.servicecontrol.v1.QuotaController.AllocateQuota" [plain @156:15..156:73]
            "deadline" [plain @157:5..157:13]: "10.0" [plain @157:15..157:19]
          - mapping [block]
            "selector" [plain @158:5..158:13]: "google.api.servicecontrol.v1.ServiceController.Check" [plain @158:15..158:67]
            "deadline" [plain @159:5..159:13]: "5.0" [plain @159:15..159:18]
          - mapping [block]
            "selector" [plain @160:5..160:13]: "google.api.servicecontrol.v1.ServiceController.Report" [plain @160:15..160:68]
            "deadline" [plain @161:5..161:13]: "16.0" [plain @161:15..161:19]
      "authentication" [plain @163:1..163:15]: mapping [block]
        "rules" [plain @164:3..164:8]: sequence [block]
          - mapping [block]
            "selector" [plain @165:5..165:13]: "google.api.servicecontrol.v1.QuotaController.AllocateQuota" [plain @165:15..165:73]
            "oauth" [plain @166:5..166:10]: mapping [block]
              "canonical_scopes" [plain @167:7..167:23]: literal [@167:25..170:1]
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/servicecontrol"
          - mapping [block]
            "selector" [plain @170:5..170:13]: "google.api.servicecontrol.v1.ServiceController.Check" [plain @170:15..170:67]
            "oauth" [plain @171:5..171:10]: mapping [block]
              "canonical_scopes" [plain @172:7..172:23]: literal [@172:25..175:1]
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/servicecontrol"
          - mapping [block]
            "selector" [plain @175:5..175:13]: "google.api.servicecontrol.v1.ServiceController.Report" [plain @175:15..175:68]
            "oauth" [plain @176:5..176:10]: mapping [block]
              "canonical_scopes" [plain @177:7..177:23]: literal [@177:25..180:1]
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/servicecontrol"
    "###);
}
