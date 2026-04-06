mod current_identity;
mod request_authenticator;

pub use self::{
    current_identity::CurrentIdentity,
    request_authenticator::{AuthenticationOutcome, RequestAuthenticator},
};
