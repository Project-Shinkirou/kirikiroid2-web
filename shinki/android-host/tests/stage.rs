use std::fs;

#[test]
fn stages_android_host_without_the_native_adapter() {
    let output = tempfile::tempdir().unwrap();
    shinki_kirikiroid2_web_android_host::stage(output.path()).unwrap();
    let output = output.path();

    let activity = fs::read_to_string(output.join("java/org/krkrsd/app/SDLActivity.java")).unwrap();
    assert!(activity.contains("package org.krkrsd.app;"));
    let helper =
        fs::read_to_string(output.join("java/org/krkrc2dx/lib/Cocos2dxHelper.java")).unwrap();
    assert!(helper.contains("org.shinkirou.krkr.KrkrHost.requestExit();"));
    let aidl = fs::read_to_string(output.join("java/com/enhance/gameservice/IGameTuningService.aidl"))
        .unwrap();
    assert!(aidl.contains("interface IGameTuningService"));
    let jar = fs::read(output.join("libs/com.android.vending.expansion.zipfile.jar")).unwrap();
    assert!(jar.starts_with(b"PK\x03\x04"));
    for notice in [
        "LICENSE_cocos2d-x.txt",
        "LICENSE_cocosdenshion.txt",
        "LICENSE_com.android.vending.expansion.zipfile.txt",
        "SDL-LICENSE.txt",
    ] {
        assert!(fs::metadata(output.join("licenses").join(notice)).unwrap().len() > 0);
    }
    let contract = fs::read(output.join("jni-contract.json")).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_slice(&contract).unwrap();
    for (class, name, signature) in [
        (
            "Lorg/krkrsd/app/SDLActivity;",
            "nativeRunMain",
            "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/Object;)I",
        ),
        ("Lorg/shinkirou/krkr/KrkrHost;", "nativeInitialize", "()I"),
    ] {
        assert!(entries.iter().any(|entry| {
            entry["class"] == class
                && entry["name"] == name
                && entry["signature"] == signature
                && entry["native"] == true
        }));
    }
}
