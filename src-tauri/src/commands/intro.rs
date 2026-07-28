use crate::services::intro::{self, IntroPublicPayload};

#[tauri::command]
pub async fn get_intro_public_data() -> IntroPublicPayload {
    intro::get_intro_public_data().await
}
