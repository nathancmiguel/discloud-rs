//! Subdomains, custom domains, snapshots, env vars, projects, databases, teams.

mod common;

use common::{mock, setup};
use discloud_rs::{
    AddAppToProject, CreateDatabase, CreateProject, CreateTeam, DatabaseBackupDetail,
    DatabaseBackupQuery, DatabaseSpec, Error, RestoreDatabase, SnapshotQuery, UpdateDatabase,
    UpdateTeam,
};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{body_json, method, path, query_param},
};

#[tokio::test]
async fn subdomains() {
    let (server, client) = setup().await;
    let sub = json!({"id": "mybot", "userID": "1", "status": 2, "date": 1719010867306u64});
    mock(
        &server,
        "GET",
        "/subdomain/all",
        200,
        json!({"status": "ok", "subdomains": [sub]}),
    )
    .await;
    mock(
        &server,
        "GET",
        "/subdomain/mybot",
        200,
        json!({"status": "ok", "subdomain": sub}),
    )
    .await;
    mock(
        &server,
        "POST",
        "/subdomain/new-one",
        201,
        json!({"status": "ok", "subdomain": {"id": "new-one", "status": 1}}),
    )
    .await;
    mock(
        &server,
        "DELETE",
        "/subdomain/mybot",
        409,
        json!({"code": 409, "message": "Subdomain in use"}),
    )
    .await;

    assert!(client.get_subdomains().await.unwrap()[0].in_use());
    assert_eq!(client.get_subdomain("mybot").await.unwrap().user_id, "1");
    assert!(!client.create_subdomain("new-one").await.unwrap().in_use());
    assert!(matches!(
        client.delete_subdomain("mybot").await,
        Err(Error::Conflict(_))
    ));
}

#[tokio::test]
async fn custom_domains() {
    let (server, client) = setup().await;
    let domain = json!({"id": "example.com", "dns_status": "pending_check", "date": 1, "isLegacy": false,
        "app": {"id": "1", "name": "site", "online": true, "type": 1},
        "instructions": [{"type": "CNAME", "name": "example.com", "content": "1.discloud.app"}]});
    mock(
        &server,
        "GET",
        "/customdomain/all",
        200,
        json!({"status": "ok", "domains": [domain]}),
    )
    .await;
    mock(
        &server,
        "GET",
        "/customdomain/example.com",
        200,
        json!({"status": "ok", "domain": domain}),
    )
    .await;
    Mock::given(method("POST"))
        .and(path("/customdomain/create"))
        .and(body_json(
            json!({"domainName": "example.com", "appID": "1"}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "domain": domain})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/customdomain/example.com/edit"))
        .and(body_json(json!({"newAppID": "2"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"status": "ok", "message": "moved", "migrated": true, "domain": "example.com"}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    mock(
        &server,
        "DELETE",
        "/customdomain/example.com/remove",
        200,
        json!({"status": "ok", "domain": "example.com"}),
    )
    .await;
    mock(&server, "GET", "/customdomain/example.com/verify", 200,
        json!({"status": "ok", "domain": {"id": "example.com", "dns_status": "active", "valid": true, "message": "ok"}})).await;

    assert_eq!(
        client.get_custom_domains().await.unwrap()[0].instructions[0].r#type,
        "CNAME"
    );
    assert_eq!(
        client
            .get_custom_domain("example.com")
            .await
            .unwrap()
            .is_legacy,
        Some(false)
    );
    assert_eq!(
        client
            .create_custom_domain("example.com", "1")
            .await
            .unwrap()
            .app
            .unwrap()
            .r#type,
        Some(1)
    );
    assert!(
        client
            .edit_custom_domain("example.com", "2")
            .await
            .unwrap()
            .migrated
    );
    client.remove_custom_domain("example.com").await.unwrap();
    assert_eq!(
        client
            .verify_custom_domain("example.com")
            .await
            .unwrap()
            .valid,
        Some(true)
    );
    assert!(client.get_custom_domain("create").await.is_err());
}

#[tokio::test]
async fn snapshots() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/snapshot"))
        .and(query_param("page", "2"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"status": "ok", "page": 2, "limit": 10, "total": 0, "backups": []}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/snapshot")).and(query_param("appID", "123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "app": {"id": "123"},
            "versions": [{"version": "20240101-120000", "size": 2048, "createdAt": "2024-01-01T12:00:00Z"}]})))
        .expect(1).mount(&server).await;
    mock(
        &server,
        "GET",
        "/snapshot/123",
        200,
        json!({"status": "ok", "app": {"id": "123"},
        "versions": [{"version": "20240101-120000", "size": 2048, "date": 1704110400000u64}]}),
    )
    .await;
    mock(&server, "POST", "/snapshot/123", 200, json!({"status": "ok", "app": {"id": "123"}, "snapshot": {
        "version": "20240101-120000", "size": "2.5 MB", "url": "https://s",
        "allVersions": [{"version": "20240101-120000", "size": "2.5 MB", "date": "2024-01-01T12:00:00Z"}]}})).await;
    mock(&server, "GET", "/snapshot/123/versions/20240101-120000", 200, json!({"status": "ok", "app": {"id": "123"},
        "download": {"url": "https://d", "expiresAt": "2024-01-01T13:00:00Z", "version": "20240101-120000", "size": 2048}})).await;
    mock(&server, "DELETE", "/snapshot/123/versions/20240101-120000", 200,
        json!({"status": "ok", "app": {"id": "123"}, "deleted": "snapshots/123/20240101-120000.zip"})).await;

    let page = client
        .get_snapshots(&SnapshotQuery {
            page: Some(2),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!((page.page, page.total), (Some(2), Some(0)));
    let filtered = client
        .get_snapshots(&SnapshotQuery {
            app_id: Some("123".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(filtered.app_id.as_deref(), Some("123"));
    assert_eq!(
        filtered.versions[0].created_at.as_deref(),
        Some("2024-01-01T12:00:00Z")
    );
    assert_eq!(
        client.get_app_snapshots("123").await.unwrap()[0].date,
        Some(1_704_110_400_000)
    );
    assert_eq!(
        client.create_snapshot("123").await.unwrap().all_versions[0].size,
        "2.5 MB"
    );
    assert_eq!(
        client
            .get_snapshot_download("123", "20240101-120000")
            .await
            .unwrap()
            .size,
        Some(2048)
    );
    assert!(
        client
            .delete_snapshot("123", "20240101-120000")
            .await
            .unwrap()
            .ends_with(".zip")
    );
    assert!(
        client
            .get_snapshots(&SnapshotQuery {
                limit: Some(500),
                ..Default::default()
            })
            .await
            .is_err()
    );
}

#[tokio::test]
async fn env_vars() {
    let (server, client) = setup().await;
    mock(
        &server,
        "GET",
        "/env/account",
        200,
        json!({"status": "ok", "data": {"vars": {"A": "1"}}}),
    )
    .await;
    Mock::given(method("PUT"))
        .and(path("/env/app/7"))
        .and(body_json(json!({"vars": {"A": "1", "B": "2"}})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status": "ok", "data": {"vars": {"A": "1", "B": "2"}}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    mock(
        &server,
        "DELETE",
        "/env/project/p1",
        200,
        json!({"status": "ok", "data": {"removed": 3}}),
    )
    .await;
    mock(
        &server,
        "GET",
        "/env/project/p1",
        403,
        json!({"status": "error", "message": "config_app permission required"}),
    )
    .await;

    assert_eq!(client.get_account_env().await.unwrap()["A"], "1");
    assert_eq!(
        client
            .set_app_env("7", [("B", "2"), ("A", "1")])
            .await
            .unwrap()
            .len(),
        2
    );
    assert_eq!(client.delete_project_env("p1").await.unwrap(), 3);
    assert!(matches!(
        client.get_project_env("p1").await,
        Err(Error::Forbidden(_))
    ));
}

#[tokio::test]
async fn projects() {
    let (server, client) = setup().await;
    Mock::given(method("POST")).and(path("/projects"))
        .and(body_json(json!({"name": "Prod", "fromAppIDs": ["1"]})))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"status": "ok", "message": "created",
            "data": {"_id": "proj1", "ownerID": "u", "name": "Prod", "description": "", "teamID": null, "clusterName": "fenrir"}})))
        .expect(1).mount(&server).await;
    mock(&server, "GET", "/projects", 200, json!({"status": "ok", "data": [{"_id": "proj1", "name": "Prod", "teamID": null, "clusterName": "fenrir"}]})).await;
    mock(
        &server,
        "GET",
        "/projects/proj1",
        200,
        json!({"status": "ok", "data": {
        "project": {"_id": "proj1", "name": "Prod", "memberRole": "owner"}, "team": null,
        "apps": [{"_id": "1", "nameTAG": "bot"}]}}),
    )
    .await;
    Mock::given(method("PATCH"))
        .and(path("/projects/proj1/team"))
        .and(body_json(json!({"teamID": null})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status": "ok", "data": {"_id": "proj1", "teamID": null}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT")).and(path("/projects/proj1/apps/1")).and(body_json(json!({"migrate": false})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "data": {"_id": "1", "projectID": "proj1", "clusterName": "fenrir"}})))
        .expect(1).mount(&server).await;
    Mock::given(method("PUT")).and(path("/projects/proj1/apps/2")).and(body_json(json!({"migrate": true})))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({"status": "accepted", "code": "MIGRATION_STARTED",
            "data": {"appID": "2", "projectID": "proj1", "from": "a", "to": "b", "estimatedSec": 30}})))
        .expect(1).mount(&server).await;
    mock(
        &server,
        "PUT",
        "/projects/proj1/apps/3",
        409,
        json!({"status": "error", "code": "CLUSTER_MISMATCH", "message": "different cluster",
        "data": {"appCluster": "a", "projectCluster": "b"}}),
    )
    .await;
    mock(
        &server,
        "DELETE",
        "/projects/proj1/apps/1",
        200,
        json!({"status": "ok", "message": "removed"}),
    )
    .await;
    mock(
        &server,
        "GET",
        "/projects/proj1/apps",
        200,
        json!({"status": "ok", "data": [{"_id": "4", "nameTAG": "x", "clusterName": "a"}]}),
    )
    .await;

    let new = CreateProject {
        name: "Prod".into(),
        from_app_ids: vec!["1".into()],
        ..Default::default()
    };
    assert_eq!(
        client
            .create_project(&new)
            .await
            .unwrap()
            .cluster_name
            .as_deref(),
        Some("fenrir")
    );
    assert_eq!(client.get_projects().await.unwrap()[0].team_id, None);
    let details = client.get_project("proj1").await.unwrap();
    assert_eq!(
        (
            details.project.member_role.as_deref(),
            details.apps[0].name_tag.as_str()
        ),
        (Some("owner"), "bot")
    );
    client.set_project_team("proj1", None).await.unwrap();
    assert!(
        matches!(client.add_app_to_project("proj1", "1", false).await.unwrap(), AddAppToProject::Added(l) if l.project_id == "proj1")
    );
    assert!(
        matches!(client.add_app_to_project("proj1", "2", true).await.unwrap(), AddAppToProject::MigrationStarted(m) if m.estimated_sec == Some(30))
    );
    let err = client
        .add_app_to_project("proj1", "3", false)
        .await
        .unwrap_err();
    assert_eq!(
        err.api_error().unwrap().code.as_deref(),
        Some("CLUSTER_MISMATCH")
    );
    client.remove_app_from_project("proj1", "1").await.unwrap();
    assert_eq!(
        client.get_project_available_apps("proj1").await.unwrap()[0].id,
        "4"
    );
}

#[tokio::test]
async fn databases() {
    let (server, client) = setup().await;
    mock(&server, "GET", "/database/presets", 200, json!({"status": "ok", "data": {"databases": [{
        "type": "postgres", "versions": ["17"], "extensions": ["pgvector"], "defaultPort": 5432,
        "suggestedDisplayName": "db", "suggestedDbName": "db", "suggestedUser": "u", "suggestedPassword": "p",
        "vars": {"required": ["POSTGRES_PASSWORD"], "optional": [], "fromSuggestions": {"POSTGRES_USER": "suggestedUser"}}}]}})).await;
    Mock::given(method("GET")).and(path("/database/presets/random-names")).and(query_param("type", "redis"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "data": {"suggestedDisplayName": "a", "suggestedDbName": "b", "suggestedUser": "c"}})))
        .expect(1).mount(&server).await;
    mock(&server, "GET", "/database", 200, json!({"status": "ok", "data": [{"_id": "x1db", "userID": "u", "displayName": "db",
        "projectID": null, "db": {"type": "postgres", "version": "17", "port": 5432}, "clusterName": "a",
        "totalRam": 256, "hostname": "db1", "addedAtTimestamp": 1}]})).await;
    Mock::given(method("POST")).and(path("/database"))
        .and(body_json(json!({"displayName": "db", "db": {"type": "postgres", "version": "17"}, "totalRam": 256, "vars": {"POSTGRES_PASSWORD": "s"}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "id": "x2db", "message": "provisioning"})))
        .expect(1).mount(&server).await;
    mock(&server, "GET", "/database/x1db", 200, json!({"status": "ok", "message": "", "database": {
        "_id": "x1db", "projectID": null, "clusterName": "a", "hostname": "db1", "displayName": "db",
        "dbType": "postgresql", "dbEngine": "postgres", "dbVersion": "17", "extensions": [], "totalRam": 256,
        "owner": {"id": "u", "username": "me", "avatar": null}, "perms": null, "vars": {"POSTGRES_USER": "u"},
        "online": true, "exists": true, "totalUsedRam": 187.3, "last_restart": "1 hour", "dataVolumeSizeHuman": "12 MB"}})).await;
    Mock::given(method("PATCH"))
        .and(path("/database/x1db"))
        .and(body_json(json!({"totalRam": 512})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "rebuilding": true,
            "data": {"_id": "x1db", "displayName": "db", "totalRam": 512, "hostname": "db1"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    for action in ["start", "stop", "restart"] {
        mock(
            &server,
            "PUT",
            &format!("/database/x1db/{action}"),
            200,
            json!({"status": "ok", "message": action}),
        )
        .await;
    }
    Mock::given(method("POST")).and(path("/database/x1db/backup")).and(body_json(json!({"gzip": true})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "data": {"backupId": 20240101120000u64, "downloadUrl": "https://b"}})))
        .expect(1).mount(&server).await;
    Mock::given(method("GET")).and(path("/database/x1db/backups")).and(query_param("limit", "5")).and(query_param("detail", "full"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "data": {
            "backups": [{"backupId": 20240101120000u64, "createdAt": "2024-01-01T12:00:00Z", "size": 10}], "nextCursor": null}})))
        .expect(1).mount(&server).await;
    Mock::given(method("GET")).and(path("/database/x1db/backups/20240101120000/download")).and(query_param("expiresInDays", "3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok", "data": {"downloadUrl": "https://d", "expiresAt": "2024-01-04T00:00:00Z"}})))
        .expect(1).mount(&server).await;
    Mock::given(method("POST"))
        .and(path("/database/x1db/restore"))
        .and(body_json(
            json!({"backupId": "20240101120000", "totalRam": 256, "db": {"version": "17"}}),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status": "ok", "id": "x3db", "message": "restoring"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    mock(&server, "GET", "/database/x1db/logs", 200, json!({"status": "ok", "app": {"_id": "x1db", "terminal": {"big": "b", "small": "s", "url": null}}})).await;
    mock(
        &server,
        "DELETE",
        "/database/x1db/logs",
        200,
        json!({"status": "ok"}),
    )
    .await;
    mock(
        &server,
        "DELETE",
        "/database/x1db",
        429,
        json!({"status": "error", "message": "Another process is already running."}),
    )
    .await;

    let presets = client.get_database_presets().await.unwrap();
    assert_eq!(
        presets[0].vars.from_suggestions["POSTGRES_USER"],
        "suggestedUser"
    );
    assert_eq!(
        client
            .get_random_database_names(Some("redis"))
            .await
            .unwrap()
            .suggested_user,
        "c"
    );
    assert_eq!(client.get_databases().await.unwrap()[0].db.port, Some(5432));

    let new = CreateDatabase {
        display_name: "db".into(),
        db: DatabaseSpec {
            kind: "postgres".into(),
            version: "17".into(),
            ..Default::default()
        },
        total_ram: 256,
        vars: [("POSTGRES_PASSWORD".to_owned(), "s".to_owned())].into(),
        ..Default::default()
    };
    assert_eq!(client.create_database(&new).await.unwrap(), "x2db");
    let db = client.get_database("x1db").await.unwrap();
    assert_eq!(
        (db.total_used_ram, db.owner.unwrap().username.as_deref()),
        (Some(187.3), Some("me"))
    );
    let updated = client
        .update_database(
            "x1db",
            &UpdateDatabase {
                total_ram: Some(512),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(updated.rebuilding);
    client.start_database("x1db").await.unwrap();
    client.stop_database("x1db").await.unwrap();
    client.restart_database("x1db").await.unwrap();
    assert_eq!(
        client
            .backup_database("x1db", true)
            .await
            .unwrap()
            .backup_id,
        "20240101120000"
    );
    let page = client
        .get_database_backups(
            "x1db",
            &DatabaseBackupQuery {
                limit: Some(5),
                detail: Some(DatabaseBackupDetail::Full),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(
        (page.backups[0].backup_id.as_str(), page.next_cursor),
        ("20240101120000", None)
    );
    client
        .get_database_backup_download("x1db", "20240101120000", Some(3))
        .await
        .unwrap();
    let restore = RestoreDatabase {
        backup_id: "20240101120000".into(),
        total_ram: 256,
        db_version: Some("17".into()),
        ..Default::default()
    };
    assert_eq!(
        client.restore_database("x1db", &restore).await.unwrap(),
        "x3db"
    );
    assert_eq!(client.get_database_logs("x1db").await.unwrap().url, None);
    client.delete_database_logs("x1db").await.unwrap();
    assert!(matches!(
        client.delete_database("x1db").await,
        Err(Error::RateLimited(_))
    ));
}

#[tokio::test]
async fn teams() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/teams"))
        .and(body_json(json!({"name": "Core"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            json!({"status": "ok", "data": {"_id": "team1", "ownerID": "u", "name": "Core"}}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    mock(
        &server,
        "GET",
        "/teams",
        200,
        json!({"status": "ok", "data": [{"_id": "team1", "name": "Core", "isOwner": true}]}),
    )
    .await;
    mock(
        &server,
        "GET",
        "/teams/apps",
        200,
        json!({"status": "ok", "data": [{"_id": "1", "nameTAG": "bot", "teamID": "team1"}]}),
    )
    .await;
    mock(
        &server,
        "GET",
        "/teams/roles/viewer/permissions",
        200,
        json!({"status": "ok", "data": {"role": "viewer", "permissions": ["view_logs"]}}),
    )
    .await;
    mock(&server, "GET", "/teams/team1", 200, json!({"status": "ok", "data": {"team": {"_id": "team1", "name": "Core"},
        "members": [{"userID": "u2", "role": "developer"}], "projects": [{"_id": "proj1", "name": "Prod"}]}})).await;
    Mock::given(method("PATCH"))
        .and(path("/teams/team1"))
        .and(body_json(json!({"description": "d"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status": "ok", "data": {"_id": "team1", "name": "Core"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    mock(
        &server,
        "DELETE",
        "/teams/team1",
        200,
        json!({"status": "ok", "message": "deleted"}),
    )
    .await;
    Mock::given(method("POST"))
        .and(path("/teams/team1/members"))
        .and(body_json(json!({"userID": "u3", "role": "viewer"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            json!({"status": "ok", "data": {"userID": "u3", "role": "viewer", "username": "x"}}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    mock(
        &server,
        "GET",
        "/teams/team1/members",
        200,
        json!({"status": "ok", "data": [{"userID": "u3", "role": "viewer", "username": "x"}]}),
    )
    .await;
    Mock::given(method("PATCH"))
        .and(path("/teams/team1/members/u3"))
        .and(body_json(json!({"role": "admin"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"status": "ok", "data": {"userID": "u3", "role": "admin"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    mock(
        &server,
        "DELETE",
        "/teams/team1/members/u3",
        200,
        json!({"status": "ok", "message": "removed"}),
    )
    .await;
    mock(
        &server,
        "POST",
        "/teams/team1/leave",
        403,
        json!({"status": "error", "message": "The owner cannot leave"}),
    )
    .await;

    assert_eq!(
        client
            .create_team(&CreateTeam {
                name: "Core".into(),
                description: None
            })
            .await
            .unwrap()
            .id,
        "team1"
    );
    assert_eq!(client.get_teams().await.unwrap()[0].is_owner, Some(true));
    assert_eq!(client.get_teams_apps().await.unwrap()[0].team_id, "team1");
    assert_eq!(
        client
            .get_team_role_permissions("viewer")
            .await
            .unwrap()
            .permissions,
        ["view_logs"]
    );
    let details = client.get_team("team1").await.unwrap();
    assert_eq!(
        (
            details.members[0].role.as_str(),
            details.projects[0].name.as_str()
        ),
        ("developer", "Prod")
    );
    client
        .update_team(
            "team1",
            &UpdateTeam {
                description: Some("d".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    client.delete_team("team1").await.unwrap();
    assert_eq!(
        client
            .add_team_member("team1", "u3", "viewer")
            .await
            .unwrap()
            .username
            .as_deref(),
        Some("x")
    );
    assert_eq!(client.get_team_members("team1").await.unwrap().len(), 1);
    assert_eq!(
        client
            .update_team_member("team1", "u3", "admin")
            .await
            .unwrap()
            .role,
        "admin"
    );
    client.remove_team_member("team1", "u3").await.unwrap();
    assert!(matches!(
        client.leave_team("team1").await,
        Err(Error::Forbidden(_))
    ));
}
