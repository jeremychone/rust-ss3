**NOTE:** **Experimental** for now, do NOT use but feel free to give feedback and cherry-pick what you need.

This is just the initial codebase, not working, but it has the basic structure. 

Yet another S3 command-line utility, but environment variables driven, even for per profile or bucket credentials. 

Key points
- Use the official [AWS-SDK-S3](https://crates.io/crates/aws-sdk-s3)
- Environment variables driven
- Will mimic most of the official `aws s3 ...` command line (however, does not intend to be too dogmatic)
- Will eventually provide a lib as well. 

Example

```ssh
ss3 ls s3://my-bucket/ -r
```

Here is the order in which the credential will be resolved 

- First check the following `SS3_BUCKET_...` environments for the given bucket
    - `SS3_BUCKET_bucket_name_KEY_ID`
    - `SS3_BUCKET_bucket_name_KEY_SECRET`
    - `SS3_BUCKET_bucket_name_REGION`  
- Second, when `--profile profile_name`, check the following `SS3_PROFILE_...` environments
    - `SS3_PROFILE_profile_name_KEY_ID`
    - `SS3_PROFILE_profile_name_KEY_SECRET`
    - `SS3_PROFILE_profile_name_REGION`  
- Third, when `--profile profile_name`, and no profile environments, will check default AWS config files
- As as a last fallback, use the default AWS environment variables: 
    - `AWS_ACCESS_KEY_ID`
    - `AWS_SECRET_ACCESS_KEY`
    - `AWS_DEFAULT_REGION`

> NOTE: '-' characters in profile and bucket names will be replaced by '_' for environment names above.

