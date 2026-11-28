// Keep the variants and their bytes in one declaration so every new domain
// automatically participates in the pairwise regression test.
macro_rules! domain_tags {
    ($($variant:ident => ($tag:literal, $version:literal)),+ $(,)?) => {
        #[derive(Clone, Copy, Debug)]
        pub(crate) enum DomainTag {
            $($variant),+
        }

        impl DomainTag {
            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$variant),+];

            pub(crate) const fn version(self) -> u32 {
                match self {
                    $(Self::$variant => $version),+
                }
            }

            pub(crate) const fn tag(self) -> &'static str {
                match self {
                    $(Self::$variant => $tag),+
                }
            }
        }
    };
}

domain_tags! {
    Evidence => ("petchain:hash:evidence:v1", 1),
    Attachment => ("petchain:hash:attachment:v1", 1),
    ClaimDocument => ("petchain:hash:claim-document:v1", 1),
    Certificate => ("petchain:hash:certificate:v1", 1),
    MedicalRecord => ("petchain:hash:medical-record:v1", 1),
    TagId => ("petchain:hash:tag-id:v1", 1),
    MedicalRecordCommitment => ("petchain:medical-record:v1", 1),
    Custody => ("petchain.custody", 1),
}

#[cfg(test)]
mod tests {
    use super::DomainTag;

    #[test]
    fn canonical_domain_prefixes_are_pairwise_distinct() {
        for (index, left) in DomainTag::ALL.iter().enumerate() {
            if !matches!(left, DomainTag::Custody) {
                let (_, version) = left.tag().rsplit_once(":v").unwrap();
                assert_eq!(version.parse::<u32>().unwrap(), left.version());
            }
            for right in &DomainTag::ALL[index + 1..] {
                assert!(
                    !left.tag().starts_with(right.tag()),
                    "{left:?} overlaps {right:?}"
                );
                assert!(
                    !right.tag().starts_with(left.tag()),
                    "{right:?} overlaps {left:?}"
                );
            }
        }
    }
}
