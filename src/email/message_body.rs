use lettre::message::{MultiPart};

pub fn verify_email_body(origin: String, verify_id: String) -> MultiPart {
    MultiPart::alternative_plain_html(
        format!(r#"This is a verification email as your email has been changed
If this was not done by you, contact your administrator as there may be a security problem

If this was you then finish the verification process by going to ths address
{}/auth/verify_email?id={}"#, origin, verify_id),
        format!(r#"
<p>This is a verification email as your email has been changed</p>
<p>If this was not done by you, contact your administrator as there may be a security problem</p>
<p>If this was you then finish the verification process by going <a href="{}/auth/verify_email?id={}">here</a></p>
        "#, origin, verify_id)
    )
}