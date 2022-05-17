

**Yet another S3 command-line**, but environment variables are driven for per bucket or profile credentials. 

**NOTE:** **Experimental** `v0.0.5` is somewhat feature complete. 

Key points:
- Uses the official [AWS-SDK-S3](https://crates.io/crates/aws-sdk-s3) library.
- Environment variables driven credentials (per bucket, per profile, fallback to AWS CLI defaults).
- Will mimic most of the official `aws s3 ...` command line (however, does not intend to be too dogmatic)
- Will eventually provide a lib as well. 

> Note: Tested on Mac and Linux (might not work on Windows, for now, contribution welcome)

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

# list all object and prefixes (--info to display total count & size, also per extensions)
ss3 ls s3://my-bucket -r --info

# Upload single file
ss3 cp ./image-01.jpg s3://my-bucket/my-folder

# Upload full folder
ss3 cp ./ s3://my-bucket/my-folder/ -r

# Upload full folder except the *.mp4
ss3 cp ./ s3://my-bucket/my-folder/ -e "*.mp4" -r

# Upload full folder but only the *.mp4 and *.jpg
ss3 cp ./ s3://my-bucket/my-folder/ -i "*.mp4" -i "*.jpg" -r

# Download a single file to a local directory (parent dirs will be )
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

> NOTE: '-' characters in profile and bucket names will be replaced by '_' for environment names above. So a bucket name `my-bucket-001` will map to the environment variable `SS3_BUCKET_my_bucket_001_KEY_ID` ...

## Other Examples

```sh

# ls
ss3 ls s3://my-bucket

# UPLOAD - cp file to s3 dir
ss3 cp ./.test-data/to-upload/image-01.jpg s3://my-bucket

# UPLOAD - cp dir to s3 dir
ss3 cp ./.test-data/to-upload/ s3://my-bucket -r

# LIST - recursive
ss3 ls s3://my-bucket -r --info

# UPLOAD - rename
ss3 cp ./.test-data/to-upload/image-01.jpg s3://my-bucket/image-01-renamed.jpg

# UPLOAD - excludes
ss3 cp .test-data/to-upload s3://my-bucket -r -e "*.txt" --exclude "*.jpg"

# UPLOAD - includes
ss3 cp .test-data/to-upload s3://my-bucket -r -i "*.txt"

# UPLOAD - cp dir to s3 (recursive)
ss3 cp ./.test-data/to-upload/ s3://my-bucket/ggg -r

# DOWNLOAD - cp s3 file to local dir 
ss3 cp s3://my-bucket/image-01.jpg ./.test-data/downloads/

# DOWNLOAD - cp s3 file to local file (rename)
ss3 cp s3://my-bucket/image-01.jpg ./.test-data/downloads/image-01-rename.jpg

# DOWNLOAD - cp s3 folder to local dir
ss3 cp s3://my-bucket/ ./.test-data/downloads/
```