pub fn get(uri: &str) -> ureq::RequestBuilder<ureq::typestate::WithoutBody> {
	let config = ureq::config::Config::builder()
		.proxy(crate::config().config.proxy.as_ref().map(|v| ureq::Proxy::new(v).ok()).flatten())
		.build();
	
	ureq::Agent::new_with_config(config)
		.get(uri)
}