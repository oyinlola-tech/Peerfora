use uuid::Uuid;

pub struct NewAdministratorInvitation<'a> {
    pub id: Uuid,
    pub inviter_id: Uuid,
    pub email: &'a str,
    pub role_key: &'a str,
    pub token_hash: &'a str,
}
