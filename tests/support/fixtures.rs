use serde_json::{json, Value};

pub fn projects() -> Value {
  json!([
    {
      "metadata": {
        "name": "project-a",
        "id": "id-prj-a",
      }
    },
    {
      "metadata": {
        "name": "project-b",
        "id": "id-prj-b",
      }
    },
  ])
}

pub fn pipelines_project_a() -> Value {
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
        "project_id": "id-prj-a",
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
        "project_id": "id-prj-a",
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
        "project_id": "id-prj-a",
        "ppl_id": "87887fa3-ced5-4b9b-aa3c-74e65003e55a",
        "wf_id": "eb86a134-3081-406a-8ca1-d6e376cf9a65"
      }
    ])
}

pub fn pipelines_project_b() -> Value {
    json!([
      {
        "name": "deploy",
        "state": "DONE",
        "result": "PASSED",
        "created_at": {
          "seconds": 1742818065,
          "nanos": 343587000
        },
        "done_at": {
          "seconds": 1742818205,
          "nanos": 730065000
        },
        "project_id": "id-prj-b",
        "ppl_id": "56c59ffa-f88d-4946-a5d0-586c2ca8a659",
        "wf_id": "f74afa10-1250-4e0e-bbed-c41c3435be62"
      },
      {
        "name": "build",
        "state": "DONE",
        "result": "PASSED",
        "created_at": {
          "seconds": 1742817972,
          "nanos": 836498000
        },
        "done_at": {
          "seconds": 1742818063,
          "nanos": 263184000
        },
        "project_id": "id-prj-b",
        "ppl_id": "d58aa83e-6f1c-418c-9176-989ec20c2d62",
        "wf_id": "f74afa10-1250-4e0e-bbed-c41c3435be62"
      }
    ])
}
