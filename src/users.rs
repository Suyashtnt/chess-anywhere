// TODO: refactor app to depend on this for getting users from the database

#[derive(Debug)]
pub struct UserService {
    db: sqlx::PgPool,
}
