//! JNI bridge: exposes zkpox-core to Kotlin on Android.
//!
//! This crate compiles to `libzkpox_mobile.so` and is loaded by
//! `ZkPoxModule.kt` via `System.loadLibrary("zkpox_mobile")`.
//!
//! Cross-compile with cargo-ndk:
//!   cargo ndk -t arm64-v8a -t armeabi-v7a build --release -p zkpox-mobile
//!
//! Place output .so files in:
//!   mobile/android/app/src/main/jniLibs/{arm64-v8a,armeabi-v7a}/libzkpox_mobile.so

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};

use zkpox_core::types::*;
use zkpox_core::{prover, verifier, antispoof};

/// Java: `static native String generateProof(String pointsJson, String requestJson)`
///
/// Called from `world.zerox1.node.ZkPoxModule`.
/// Returns JSON string of `ProofResult`, or a string starting with "ERROR:" on failure.
#[no_mangle]
pub extern "system" fn Java_world_zerox1_node_ZkPoxModule_generateProofNative(
    mut env: JNIEnv,
    _class: JClass,
    points_json: JString,
    request_json: JString,
) -> jstring {
    let result = (|| -> Result<String, String> {
        let points_str: String = env
            .get_string(&points_json)
            .map_err(|e| format!("Failed to read points JSON: {e}"))?
            .into();

        let request_str: String = env
            .get_string(&request_json)
            .map_err(|e| format!("Failed to read request JSON: {e}"))?
            .into();

        let points: Vec<SignedGPSPoint> = serde_json::from_str(&points_str)
            .map_err(|e| format!("Failed to parse points: {e}"))?;

        let request: ProofRequest = serde_json::from_str(&request_str)
            .map_err(|e| format!("Failed to parse request: {e}"))?;

        // Anti-spoofing check before proof generation
        let spoof_check = antispoof::analyze(&points);
        if spoof_check.verdict == antispoof::SpoofVerdict::LikelySpoofed {
            return Err(format!(
                "GPS data appears spoofed (score: {:.2}, teleports: {}, zero-noise: {})",
                spoof_check.suspicion_score,
                spoof_check.teleport_count,
                spoof_check.zero_noise_count,
            ));
        }

        let proof_result = prover::generate_proof(&points, &request)
            .map_err(|e| format!("{e}"))?;

        serde_json::to_string(&proof_result)
            .map_err(|e| format!("Serialization failed: {e}"))
    })();

    let output = match result {
        Ok(json) => json,
        Err(e) => format!("ERROR:{e}"),
    };

    env.new_string(&output)
        .unwrap_or_else(|_| env.new_string("ERROR:JNI string creation failed").unwrap())
        .into_raw()
}

/// Java: `static native int verifyProofNative(String resultJson)`
///
/// Returns 1 for valid, 0 for invalid, -1 for error.
#[no_mangle]
pub extern "system" fn Java_world_zerox1_node_ZkPoxModule_verifyProofNative(
    mut env: JNIEnv,
    _class: JClass,
    result_json: JString,
) -> jint {
    let outcome = (|| -> Result<bool, String> {
        let json_str: String = env
            .get_string(&result_json)
            .map_err(|e| format!("Failed to read JSON: {e}"))?
            .into();

        let proof_result: ProofResult = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse: {e}"))?;

        match verifier::verify_proof(&proof_result) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    })();

    match outcome {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
    }
}

/// Java: `static native String analyzeSpoofRisk(String pointsJson)`
///
/// Returns JSON with spoofing analysis results.
#[no_mangle]
pub extern "system" fn Java_world_zerox1_node_ZkPoxModule_analyzeSpoofRiskNative(
    mut env: JNIEnv,
    _class: JClass,
    points_json: JString,
) -> jstring {
    let result = (|| -> Result<String, String> {
        let json_str: String = env
            .get_string(&points_json)
            .map_err(|e| format!("Failed to read JSON: {e}"))?
            .into();

        let points: Vec<SignedGPSPoint> = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse points: {e}"))?;

        let analysis = antispoof::analyze(&points);

        let json = serde_json::json!({
            "total_points": analysis.total_points,
            "teleport_count": analysis.teleport_count,
            "zero_noise_count": analysis.zero_noise_count,
            "impossible_velocity_count": analysis.impossible_velocity_count,
            "suspicion_score": analysis.suspicion_score,
            "verdict": format!("{:?}", analysis.verdict),
        });

        Ok(json.to_string())
    })();

    let output = match result {
        Ok(json) => json,
        Err(e) => format!("{{\"error\":\"{e}\"}}"),
    };

    env.new_string(&output)
        .unwrap_or_else(|_| env.new_string("{\"error\":\"JNI error\"}").unwrap())
        .into_raw()
}
