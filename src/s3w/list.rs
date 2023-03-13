use super::sitem::SItem;
use super::{validate_key, SBucket};
use crate::Error;
use globset::GlobSet;

// region:    --- ListOptions
pub enum ListInfo {
	WithInfo,
	InfoOnly,
}

#[derive(Default)]
pub struct ListOptions {
	pub recursive: bool, // default will be false by Default
	pub excludes: Option<GlobSet>,
	pub includes: Option<GlobSet>,
	pub continuation_token: Option<String>,
	pub info: Option<ListInfo>,
}

impl ListOptions {
	pub fn new(recursive: bool) -> ListOptions {
		ListOptions {
			recursive,
			..Default::default()
		}
	}
}
pub struct ListResult {
	pub prefixes: Vec<SItem>,
	pub objects: Vec<SItem>,
	pub next_continuation_token: Option<String>,
}
// endregion: --- ListResult

impl SBucket {
	pub async fn list(&self, prefix: &str, options: &ListOptions) -> Result<ListResult, Error> {
		// BUILD - the aws S3 list request
		let mut builder = self
			.client
			.list_objects_v2()
			.prefix(prefix)
			.bucket(&self.name)
			.set_continuation_token(options.continuation_token.clone());

		if !options.recursive {
			builder = builder.delimiter("/");
		}

		// EXECUTE - the AWS S3 request
		let resp = match builder.send().await {
			Ok(resp) => resp,
			Err(err) => Err(err)?,
		};

		// get the prefixes
		let prefixes: Vec<SItem> = resp
			.common_prefixes()
			.unwrap_or_default()
			.iter()
			.filter(|o| {
				o.prefix()
					.map(|p| validate_key(p, &options.includes, &options.excludes))
					.unwrap_or(false)
			})
			.map(SItem::from_prefix)
			.collect();

		// get the objects
		let objects: Vec<SItem> = resp
			.contents()
			.unwrap_or_default()
			.iter()
			.filter(|o| {
				o.key()
					.map(|p| validate_key(p, &options.includes, &options.excludes))
					.unwrap_or(false)
			})
			.map(SItem::from_object)
			.collect();

		let next_continuation_token = resp.next_continuation_token().map(|t| t.to_string());

		Ok(ListResult {
			prefixes,
			objects,
			next_continuation_token,
		})
	}
}
