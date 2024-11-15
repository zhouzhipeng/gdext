/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use godot::obj::NewGd;
use godot::register::GodotClass;
use godot::tools::{load, save, try_load, try_save};

use crate::framework::itest;

fn remove_test_file(file_name: &str) {
    let godot_path = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../godot/"));
    let file_path = godot_path.join(file_name);
    std::fs::remove_file(&file_path)
        .unwrap_or_else(|_| panic!("couldn't remove test file: {}", file_path.display()));
}

#[derive(GodotClass)]
#[class(base=Resource, init)]
struct SavedGame {
    #[export]
    level: u32,
}

const RESOURCE_NAME: &str = "test_resource.tres";
const FAULTY_PATH: &str = "no_such_path";

#[itest]
fn save_test() {
    let res_path = format!("res://{}", RESOURCE_NAME);

    let resource = SavedGame::new_gd();

    let res = try_save(&resource, FAULTY_PATH);
    assert!(res.is_err());

    let res = try_save(&resource, &res_path);
    assert!(res.is_ok());

    save(&resource, &res_path);

    remove_test_file(RESOURCE_NAME);
}

#[itest]
fn load_test() {
    let level = 2317;
    let res_path = format!("res://{}", RESOURCE_NAME);

    let mut resource = SavedGame::new_gd();
    resource.bind_mut().set_level(level);

    save(&resource, &res_path);

    let res = try_load::<SavedGame>(FAULTY_PATH);
    assert!(res.is_err());

    let res = try_load::<SavedGame>(&res_path);
    assert!(res.is_ok());

    let loaded = res.unwrap();
    assert_eq!(loaded.bind().get_level(), level);

    let loaded = load::<SavedGame>(&res_path);
    assert_eq!(loaded.bind().get_level(), level);

    remove_test_file(RESOURCE_NAME);
}
