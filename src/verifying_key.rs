use ed25519_dalek::VerifyingKey;
use worker::Env;

pub(crate) fn verifying_key(env: &Env,env_key: &str) -> VerifyingKey {
  let vk = env.var(env_key).unwrap().to_string();
  let mut bytes = [0; 32];
  hex::decode_to_slice(vk, &mut bytes).unwrap();
  VerifyingKey::from_bytes(&bytes).unwrap()
}