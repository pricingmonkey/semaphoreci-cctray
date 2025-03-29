use serde_json::{json, Value};

pub fn pipeline_response_body() -> Value {
    json!([
      {
        "name": "build",
        "state": "RUNNING",
        "created_at": {
          "seconds": 1743180511,
          "nanos": 682810000
        },
        "done_at": {
          "seconds": 0,
          "nanos": 0
        },
        "ppl_id": "0a3e10c1-f046-4959-ae9d-2677a997a72c",
        "wf_id": "94505eb4-27d2-4d5c-a616-27077ae9ac32"
      },
      {
        "name": "deploy",
        "state": "DONE",
        "result": "FAILED",
        "created_at": {
          "seconds": 1743180245,
          "nanos": 651338000
        },
        "done_at": {
          "seconds": 1743180510,
          "nanos": 558706000
        },
        "ppl_id": "7ba0d874-33f0-4495-af7c-8cbccb7f56e5",
        "wf_id": "eb86a134-3081-406a-8ca1-d6e376cf9a65"
      },
      {
        "name": "build",
        "state": "DONE",
        "result": "PASSED",
        "created_at": {
          "seconds": 1742826854,
          "nanos": 852498000
        },
        "done_at": {
          "seconds": 1742826923,
          "nanos": 771318000
        },
        "ppl_id": "87887fa3-ced5-4b9b-aa3c-74e65003e55a",
        "wf_id": "eb86a134-3081-406a-8ca1-d6e376cf9a65"
      }
    ])
}
