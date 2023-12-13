use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const HEADER_FILE: &str = "cpc_nvm3.h";

fn find_target_dir(out_dir: &Path) -> Option<&Path> {
    out_dir.parent()?.parent()?.parent()
}

fn main() {
    let crate_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let header = out_dir.join(HEADER_FILE);

    let mut config: cbindgen::Config = Default::default();
    config.language = cbindgen::Language::C;
    config.documentation = true;
    config.documentation_length = cbindgen::DocumentationLength::Full;
    config.include_guard = Some("CPC_NVM3_H".to_string());
    config.export.include = vec!["CpcNvm3ErrorCodes".to_string()];
    config.export.exclude = vec![String::from("cpc_deinit")];
    config.sys_includes = vec![String::from("stdio.h")];
    config.header = Some(
        "/*******************************************************************************
* @file
* @brief Co-Processor Communication Protocol(CPC) NVM3 - Library Header
*******************************************************************************
* # License
* <b>Copyright 2023 Silicon Laboratories Inc. www.silabs.com</b>
*******************************************************************************
*
* The licensor of this software is Silicon Laboratories Inc. Your use of this
* software is governed by the terms of Silicon Labs Master Software License
* Agreement (MSLA) available at
* www.silabs.com/about-us/legal/master-software-license-agreement. This
* software is distributed to you in Source Code format and is governed by the
* sections of the MSLA applicable to Source Code.
*
******************************************************************************/

/***************************************************************************/ /**
* @addtogroup cpc_nvm3 CPC_NVM3
* @{
******************************************************************************/
 "
        .to_string(),
    );

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&header);

    // Add footer
    let footer = "/** @} (end addtogroup cpc_nvm3) */\n";
    std::fs::OpenOptions::new()
        .append(true)
        .open(&header)
        .unwrap()
        .write_all(footer.as_bytes())
        .unwrap();

    if let Some(target_dir) = find_target_dir(&out_dir) {
        let to = target_dir.join(HEADER_FILE);
        fs::create_dir_all(to.parent().unwrap()).unwrap();
        fs::copy(header, to).unwrap();
    }

    // https://github.com/rust-lang/cargo/issues/5045
    // https://gitlab.kitware.com/cmake/cmake/-/issues/22307#note_971562
    println!("cargo:rustc-link-arg=-Wl,-soname,libcpc_nvm3.so");

    // https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargorerun-if-changedpath
    println!("cargo:rerun-if-changed=build.rs");
}
