//! User, applications, moderators, team apps, upload and explorer routes.

mod common;

use common::{TOKEN, mock, setup};
use discloud_rs::{AppProfileUpdate, AptPackage, FileKind, Locale, TeamPerms};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{body_json, body_string_contains, header, header_regex, method, path, query_param},
};

#[tokio::test]
async fn user_info_and_locale() {
    let (server, client) = setup().await;
    mock(
        &server,
        "GET",
        "/user",
        200,
        json!({
            "status": "ok", "message": "",
            "user": {
                "userID": "1", "username": null, "avatar": null,
                "totalRamMb": 1024, "ramUsedMb": 100,
                "subdomains": ["a"], "customdomains": [], "apps": ["2"],
                "plan": "Gold", "locale": "en-US",
                "lastDataLeft": {"days": 1, "hours": 2, "minutes": 3, "seconds": 4},
                "planDataEnd": "Subscription"
            }
        }),
    )
    .await;
    mock(
        &server,
        "PUT",
        "/locale/pt-BR",
        200,
        json!({"status": "ok", "locale": "pt-BR"}),
    )
    .await;

    let user = client.get_user_info().await.unwrap();
    assert_eq!((user.total_ram, user.used_ram), (1024, 100));
    assert_eq!(user.last_data_left.unwrap().minutes, 3);
    assert_eq!(user.username, None);
    client.set_locale(Locale::PtBR).await.unwrap();
    assert_eq!("en-US".parse::<Locale>().unwrap(), Locale::EnUS);
}

#[tokio::test]
async fn vscode_user() {
    let (server, client) = setup().await;
    mock(
        &server,
        "GET",
        "/vscode",
        200,
        json!({"status": "ok", "user": {
            "userID": "1", "plan": "Free", "locale": "en-US", "totalRamMb": 100, "ramUsedMb": 0,
            "apps": ["2"], "appsTeam": ["3"], "subdomains": [], "customdomains": [],
            "appsStatus": [{"id": "2", "name": "bot", "type": 0, "online": true, "ram": 100,
              "ramKilled": false, "exitCode": 0, "lang": "javascript", "mainFile": "index.js",
              "avatarURL": "https://x", "autoRestart": false, "apts": [], "clusterName": "fenrir", "version": "latest"}]
        }}),
    )
    .await;
    let user = client.get_vscode_user().await.unwrap();
    assert_eq!(user.apps_team, ["3"]);
    assert_eq!(user.apps_status[0].cluster_name.as_deref(), Some("fenrir"));
}

#[tokio::test]
async fn app_info_matches_spec_example() {
    let (server, client) = setup().await;
    mock(
        &server,
        "GET",
        "/app/1745338954955",
        200,
        json!({
            "status": "ok",
            "message": "The information of your application has been loaded.",
            "apps": {
                "id": "1745338954955", "avatarURL": "https://i.ibb.co/xSxd0Hv1/Uma105.png",
                "name": "Temp2 - JS", "type": 0, "online": true, "ramKilled": false,
                "exitCode": 0, "ram": 100, "mainFile": "src/index.js", "lang": "javascript",
                "autoDeployGit": "no", "autoRestart": false, "mods": [], "apts": [],
                "addedTimestamp": 1745338954955u64
            }
        }),
    )
    .await;
    mock(
        &server,
        "GET",
        "/app/all",
        200,
        json!({"status": "ok", "apps": [
            {"id": "1", "name": "a", "addedAtTimestamp": 5, "clusterName": "fenrir", "exitCode": null, "mods": null,
             "syncGit": {"autoDeploy": true, "repositoryID": 9, "branchName": "main", "lastCommit": {"message": "m", "sha": "s"}}},
            {"id": "2", "name": "b"}
        ]}),
    )
    .await;

    let app = client.get_app("1745338954955").await.unwrap();
    assert_eq!(app.added_at_timestamp, 1_745_338_954_955);
    assert_eq!(app.auto_deploy_git.as_deref(), Some("no"));
    assert_eq!(app.cluster_name, None);

    let apps = client.get_all_apps().await.unwrap();
    assert_eq!(apps.len(), 2);
    assert_eq!(apps[0].added_at_timestamp, 5);
    assert_eq!(apps[0].sync_git.as_ref().unwrap().repository_id, 9);
    assert!(apps[0].mods.is_empty());
}

#[tokio::test]
async fn status_logs_backup() {
    let (server, client) = setup().await;
    let status = json!({"id": "1", "container": "Online", "cpu": "0.00%", "memory": "39.9MB/100MB",
        "ssd": "1MB", "netIO": {"down": "5MB", "up": "1MB"}, "last_restart": "30 minutes",
        "startedAt": "2025-04-22T16:22:47.541950597Z"});
    mock(
        &server,
        "GET",
        "/app/1/status",
        200,
        json!({"status": "ok", "apps": status}),
    )
    .await;
    mock(
        &server,
        "GET",
        "/team/all/status",
        200,
        json!({"status": "ok", "apps": [status]}),
    )
    .await;
    mock(&server, "GET", "/app/1/logs", 200,
        json!({"status": "ok", "apps": {"id": "1", "terminal": {"big": "all", "small": "some", "url": "https://logs"}}})).await;
    mock(
        &server,
        "GET",
        "/team/all/logs",
        200,
        json!({"status": "ok", "apps": []}),
    )
    .await;
    mock(
        &server,
        "GET",
        "/app/all/backup",
        200,
        json!({"status": "ok", "backups": [{"id": "1", "url": "https://b", "status": "ok"}]}),
    )
    .await;
    mock(
        &server,
        "GET",
        "/team/1/backup",
        200,
        json!({"status": "ok", "backups": {"id": "1", "url": "https://b"}}),
    )
    .await;

    let s = client.get_app_status("1").await.unwrap();
    assert_eq!(
        (s.net_io.down.as_str(), s.last_restart.as_str()),
        ("5MB", "30 minutes")
    );
    assert_eq!(client.get_all_team_apps_status().await.unwrap().len(), 1);
    assert_eq!(
        client
            .get_app_logs("1")
            .await
            .unwrap()
            .terminal
            .small
            .as_deref(),
        Some("some")
    );
    assert!(client.get_all_team_apps_logs().await.unwrap().is_empty());
    assert_eq!(
        client.get_all_apps_backup().await.unwrap()[0]
            .status
            .as_deref(),
        Some("ok")
    );
    assert_eq!(client.get_team_app_backup("1").await.unwrap().status, None);
}

#[tokio::test]
async fn lifecycle_single_and_bulk() {
    let (server, client) = setup().await;
    mock(&server, "PUT", "/app/1/start", 200,
        json!({"status": "ok", "message": "started", "appStatus": {"online": true, "ramKilled": false, "exitCode": 0}})).await;
    mock(&server, "PUT", "/team/all/start", 200,
        json!({"status": "ok", "apps": {"started": ["1"], "alreadyOnline": ["2"], "alreadyInProcess": []}})).await;
    mock(&server, "PUT", "/app/all/stop", 200,
        json!({"status": "ok", "apps": {"stoped": ["1"], "alreadyOffline": [], "alreadyInProcess": ["3"]}})).await;
    mock(
        &server,
        "PUT",
        "/team/1/restart",
        200,
        json!({"status": "ok", "message": "restarted"}),
    )
    .await;
    mock(&server, "PUT", "/app/all/restart", 200,
        json!({"status": "ok", "apps": {"restarted": ["1"], "alreadyOffline": ["2"], "alreadyInProcess": []}})).await;
    mock(
        &server,
        "DELETE",
        "/app/1/delete",
        200,
        json!({"status": "ok", "message": "deleted"}),
    )
    .await;
    mock(&server, "DELETE", "/app/all/delete", 200,
        json!({"status": "ok", "apps": {"removealled": ["1"], "alreadyOffline": [], "alreadyInProcess": []}})).await;

    assert!(client.start_app("1").await.unwrap().online);
    assert_eq!(
        client.start_all_team_apps().await.unwrap().already_online,
        ["2"]
    );
    let stopped = client.stop_all_apps().await.unwrap();
    assert_eq!(
        (
            stopped.stopped.as_slice(),
            stopped.already_in_process.as_slice()
        ),
        (&["1".to_owned()][..], &["3".to_owned()][..])
    );
    client.restart_team_app("1").await.unwrap();
    assert_eq!(
        client.restart_all_apps().await.unwrap().already_offline,
        ["2"]
    );
    client.delete_app("1").await.unwrap();
    assert_eq!(client.delete_all_apps().await.unwrap().removed, ["1"]);
}

#[tokio::test]
async fn ram_profile_apt_console_exec() {
    let (server, client) = setup().await;
    Mock::given(method("PUT"))
        .and(path("/team/1/ram"))
        .and(body_json(json!({"ramMB": 256})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status": "ok", "statusCode": 200, "message": "ok"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/app/1/profile"))
        .and(body_json(json!({"name": "New name"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status": "ok", "statusCode": 200, "message": "ok"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/app/1/apt"))
        .and(body_json(json!({"apt": "ffmpeg,java"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status": "ok", "statusCode": 200, "message": "removed"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/app/1/console"))
        .and(body_json(json!({"command": "ls"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"status": "ok",
            "apps": {"id": "1", "shell": {"online": true, "stdout": "index.js\n", "stderr": ""}}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/app/1/exec"))
        .and(body_json(json!({"cmd": "node -v"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "statusCode": 200,
            "exec": {"stdout": "v22\n", "stderr": "", "exitCode": 0}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    client.set_team_app_ram("1", 256).await.unwrap();
    client
        .update_app_profile(
            "1",
            &AppProfileUpdate {
                name: Some("New name".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(
        client
            .uninstall_apt("1", &[AptPackage::Ffmpeg, AptPackage::Java])
            .await
            .unwrap(),
        "removed"
    );
    assert_eq!(
        client.run_console_command("1", "ls").await.unwrap().stdout,
        "index.js\n"
    );
    assert_eq!(
        client
            .exec_command("1", "node -v")
            .await
            .unwrap()
            .unwrap()
            .exit_code,
        Some(0)
    );
}

#[tokio::test]
async fn commit_and_upload_send_multipart() {
    let (server, client) = setup().await;
    for (verb, route) in [
        ("PUT", "/app/1/commit"),
        ("PUT", "/team/1/commit"),
        ("POST", "/upload"),
    ] {
        Mock::given(method(verb))
            .and(path(route))
            .and(header("api-token", TOKEN))
            // The boundary must be present: a bare `multipart/form-data`
            // content type makes the body unparseable.
            .and(header_regex("content-type", "^multipart/form-data; boundary=.+"))
            .and(body_string_contains("name=\"file\"; filename=\"app.zip\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "ok", "statusCode": 200, "message": "done", "logs": null,
                "app": {"id": "9", "type": 0, "name": "Bot javascript", "avatarURL": "https://i.imgur.com/bWhx7OT.png",
                        "ram": 100, "apts": "", "clusterName": "fenrir", "syncGit": null, "mainFile": "index.js",
                        "lang": "javascript", "version": "latest", "addedAtTimestamp": 1719010867306u64, "autoRestart": false}
            })))
            .expect(1)
            .mount(&server)
            .await;
    }

    let dir = std::env::temp_dir().join(format!("discloud-rs-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let zip = dir.join("app.zip");
    std::fs::write(&zip, b"PK\x05\x06fake zip").unwrap();

    assert_eq!(client.commit_app("1", &zip).await.unwrap().message, "done");
    client
        .commit_team_app_bytes("1", b"zip".to_vec(), "app.zip")
        .await
        .unwrap();
    let app = client.upload_app(&zip).await.unwrap();
    assert_eq!(
        (app.id.as_str(), app.version.as_deref()),
        ("9", Some("latest"))
    );

    std::fs::remove_dir_all(dir).ok();
}

#[tokio::test]
async fn upload_to_project_sends_project_id() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/upload"))
        .and(body_string_contains("name=\"projectId\"\r\n\r\nproj1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "app": {"id": "9"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    client
        .upload_app_bytes(b"zip".to_vec(), "app.zip", Some("proj1"))
        .await
        .unwrap();
}

#[tokio::test]
async fn missing_upload_file_is_io_error() {
    let (_server, client) = setup().await;
    let err = client
        .upload_app("definitely/missing.zip")
        .await
        .unwrap_err();
    assert!(matches!(err, discloud_rs::Error::Io(_)));
}

#[tokio::test]
async fn moderators() {
    let (server, client) = setup().await;
    mock(&server, "GET", "/app/1/team", 200, json!({"status": "ok", "team": [
        {"modID": "m1", "perms": ["start_app", "logs_app", "future_perm"], "username": "mod", "avatar": null}]})).await;
    Mock::given(method("POST"))
        .and(path("/app/1/team"))
        .and(body_json(
            json!({"modID": "m2", "perms": ["stop_app", "edit_ram"]}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"status": "ok",
            "app": {"modID": "m2", "appID": "1", "perms": ["stop_app", "edit_ram"]}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT")).and(path("/app/1/team"))
        .and(body_json(json!({"modID": "m1", "perms": ["start_app", "logs_app", "backup_app"]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok",
            "app": {"modID": "m1", "appID": "1", "perms": ["start_app", "logs_app", "backup_app"]}})))
        .expect(1).mount(&server).await;

    let team = client.get_app_team("1").await.unwrap();
    assert_eq!(team[0].app_id, "1");
    assert_eq!(
        team[0].perms,
        [TeamPerms::Start, TeamPerms::Logs, TeamPerms::Unknown]
    );
    assert_eq!(team[0].username.as_deref(), Some("mod"));

    let added = client
        .add_app_mod("1", "m2", vec![TeamPerms::Stop, TeamPerms::Ram])
        .await
        .unwrap();
    assert_eq!(added.mod_id, "m2");

    // Unknown permissions are dropped before sending.
    let edited = team[0]
        .add_perms(&client, vec![TeamPerms::Backup])
        .await
        .unwrap();
    assert!(edited.perms.contains(&TeamPerms::Backup));
}

#[tokio::test]
async fn team_apps_list() {
    let (server, client) = setup().await;
    mock(
        &server,
        "GET",
        "/team",
        200,
        json!({"status": "ok", "apps": [{
        "id": "1", "name": "bot", "type": 0, "online": false, "ram": 100, "ramKilled": false,
        "exitCode": 1, "lang": "python", "owner": {"name": "owner", "avatar": "https://a"},
        "perms": ["status_app"]}]}),
    )
    .await;
    let apps = client.get_team_apps().await.unwrap();
    assert_eq!(apps[0].owner.as_ref().unwrap().name, "owner");
    assert_eq!(apps[0].perms, [TeamPerms::Status]);
}

#[tokio::test]
async fn explorer() {
    let (server, client) = setup().await;
    Mock::given(method("GET")).and(path("/app/1/explorer")).and(query_param("cPath", "/home/node/src"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "path": "/home/node/src",
            "dir": [{"name": "index.js", "type": "file", "size": 42}, {"name": "lib", "type": "directory"}]})))
        .expect(1).mount(&server).await;
    Mock::given(method("GET"))
        .and(path("/app/1/explorer/open"))
        .and(query_param("cPath", "/a b.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"status": "ok", "path": "/a b.txt", "name": "a b.txt", "content": "hi"}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/app/1/explorer"))
        .and(body_json(json!({"typeFile": "directory", "cPath": "/new"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "path": "/new"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/app/1/explorer/edit"))
        .and(body_json(json!({"fileContent": "x", "cPath": "/f"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "path": "/f"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let listing = client
        .list_files("1", Some("/home/node/src"))
        .await
        .unwrap();
    assert_eq!(listing.entries[0].size, Some(42));
    assert_eq!(listing.entries[1].kind, FileKind::Directory);
    assert_eq!(
        client.read_file("1", "/a b.txt").await.unwrap().content,
        "hi"
    );
    assert_eq!(
        client
            .create_file("1", "/new", FileKind::Directory)
            .await
            .unwrap(),
        "/new"
    );
    assert_eq!(client.edit_file("1", "/f", "x").await.unwrap(), "/f");
}
