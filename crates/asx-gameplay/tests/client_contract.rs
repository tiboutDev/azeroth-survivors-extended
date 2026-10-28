use asx_gameplay::client_contract::*;
use asx_gameplay::Capabilities;
fn contract() -> ClientContract { ClientContract { api: API, domains: Domain::ALL.map(|domain| DomainBinding { domain, migration: Migration::Legacy }), combat: Capabilities::default(), allied_support: false, shared_snapshot: false } }
#[test]fn contract_requires_all_five_domains_and_compatible_api() {
    let c=contract();assert!(c.validate(API).is_ok());
    for d in Domain::ALL { assert_eq!(c.domain(d).domain,d); }
    let mut bad=c;bad.domains[1]=bad.domains[0];assert!(bad.validate(API).is_err());
    assert!(!API.accepts(ApiVersion{major:2,minor:0}));assert!(!API.accepts(ApiVersion{major:1,minor:1}));
    assert!(ApiVersion{major:1,minor:2}.accepts(API));
}
