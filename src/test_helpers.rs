use crate::common::etcd::EtcdPair;

pub fn assert_contains_pair(pairs: &[EtcdPair], key: &str, value: &str) {
    assert!(pairs.iter().any(|p| p.key() == key && p.value() == value));
}
