#[macro_use]
extern crate pam_sys;

use pam_sys::{PamConversation, PamHandle, PamFlag, PamReturnCode};

struct PamCustom;

impl PamConversation for PamCustom {
    fn authenticate(_pamh: PamHandle, _flags: PamFlag, _args: Vec<String>) -> PamReturnCode {
        println!("[DataJoint]: Starting...");
        // If you need login/password here, that works like this:
        //

        // let user = match _pamh.get_user(None) {
        //     Ok(Some(u)) => u,
        //     Ok(None) => return PamError::USER_UNKNOWN,
        //     Err(e) => return e,
        // };
        // println!("user: {:?}", user);
        // //
        // let pass = match _pamh.get_authtok(None) {
        //     Ok(Some(p)) => p,
        //     Ok(None) => return PamError::AUTH_ERR,
        //     Err(e) => return e,
        // };

        // println!("pass: {:?}", pass);

        PamReturnCode::SUCCESS
        // PamError::AUTH_ERR
    }
}

pam_module!(PamCustom);