

**Yet another S3 command-line**, but environment variables driven, even for per profile or bucket credentials. 

**NOTE:** **Experimental** `v0.0.2` is somewhat feature complete. 

Key points:
- Use the official [AWS-SDK-S3](https://crates.io/crates/aws-sdk-s3)
- Environment variables driven
- Will mimic most of the official `aws s3 ...` command line (however, does not intend to be too dogmatic)
- Will eventually provide a lib as well. 

> Note: Tested on Mac and Linux (might not work on Windows for now)

## Install

```sh
# With Cargo install
cargo install ss3

# Or install the binary (mac or linux) with binst (https://binst.io)
binst install ss3
```

## Command Examples

```sh
# list all object and prefixes (-r for recursive)
ss3 ls s3://my-bucket -r

# Upload single file
ss3 cp ./image-01.jpg s3://my-bucket/my-folder

# Upload full folder
ss3 cp ./ s3://my-bucket/my-folder/ -r

# Download single file to a local directory (parent dirs will be )
ss3 cp s3://my-bucket/image-01.jpg ./.downloads/

# Download a full folder (for now make sure to add end '/' in the s3 URL to distinguish from object)
ss3 cp s3://my-bucket/my-folder/ ./.downloads/ -r
```

## Configurations

Here is the order in which the credentials will be resolved:

- First check the following `SS3_BUCKET_...` environments for the given bucket
    - `SS3_BUCKET_bucket_name_KEY_ID`
    - `SS3_BUCKET_bucket_name_KEY_SECRET`
    - `SS3_BUCKET_bucket_name_REGION`  
    - `SS3_BUCKET_bucket_name_ENDPOINT` (optional for minio)     
- Second, when `--profile profile_name`, check the following `SS3_PROFILE_...` environments
    - `SS3_PROFILE_profile_name_KEY_ID`
    - `SS3_PROFILE_profile_name_KEY_SECRET`
    - `SS3_PROFILE_profile_name_REGION`  
    - `SS3_PROFILE_profile_name_ENDPOINT` (optional for minio) 
- Third, when `--profile profile_name`, and no profile environments, will check default AWS config files
- As as a last fallback, use the default AWS environment variables: 
    - `AWS_ACCESS_KEY_ID`
    - `AWS_SECRET_ACCESS_KEY`
    - `AWS_DEFAULT_REGION`

> NOTE: '-' characters in profile and bucket names will be replaced by '_' for environment names above.

