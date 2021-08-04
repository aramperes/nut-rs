use crate::proto::impl_sentences;

impl_sentences! {
    /// Server responds with the number of prior logins to the given `ups_name` device.
    RespondNumLogins (
        {
            0: NumLogins,
            1: Arg,
            2: Arg,
            3: EOL,
        },
        {
            /// The name of the UPS device.
            1: ups_name,
            /// The number of logins to the UPS device.
            2: num_logins,
        }
    ),
}

#[cfg(test)]
mod tests {
    use super::Sentences;
    use crate::proto::test_encode_decode;
    #[test]
    fn test_encode_decode() {
        test_encode_decode!(
            ["NUMLOGINS", "nutdev", "42"] <=>
            Sentences::RespondNumLogins {
                ups_name: "nutdev".into(),
                num_logins: "42".into(),
            }
        );
    }
}
