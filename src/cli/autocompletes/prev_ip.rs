use inquire::{
    autocompletion::{Autocomplete, Replacement},
    CustomUserError
};
use crate::access::prev_ip;


#[derive(Clone)]
pub struct PrevIpAutocomplete {
    prev_ip_acc: prev_ip::PrevIpAccessor
}

impl PrevIpAutocomplete {
    pub fn new(prev_ip_acc: prev_ip::PrevIpAccessor) -> Self {
        Self { prev_ip_acc }
    }
}

impl Autocomplete for PrevIpAutocomplete {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        let value = self.prev_ip_acc.get_prev_ips(input.to_owned())?;

        Ok(value)
    }

    fn get_completion(
            &mut self,
            input: &str,
            highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        let res = match highlighted_suggestion {
            Some(s) => Some(s),
            None => Some(self.prev_ip_acc.add_address(input.to_owned())?)
        };
        Ok(res)
    }
}
