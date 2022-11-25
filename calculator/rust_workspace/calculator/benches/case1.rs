use std::{collections::HashMap, fs::File, io::BufReader};

use criterion::{criterion_group, criterion_main, Criterion};
use mhr_calculator::{
    calc::{calc_data_manager::CalcDataManager, types::SkillSlotCount},
    calculate_skillset, convert_to_skills_container,
    data::{
        armor::SexType,
        data_manager::DataManager,
        parser::{parse_anomaly, parse_talisman},
    },
};
use nohash_hasher::IntMap;

fn bench(c: &mut Criterion) {
    c.bench_function("case1", |b| {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();

        let mut dm = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                DataManager::from_sources(
                    &manifest_dir.join("../src-tauri/data/armor.json"),
                    &manifest_dir.join("../src-tauri/data/skill.json"),
                    &manifest_dir.join("../src-tauri/data/deco.json"),
                )
                .await
            });

        let anomalies = parse_anomaly(
            &manifest_dir.join("../bench-data/case1/anomaly.txt"),
            dm.get_armors(),
            dm.get_armor_name_dict(),
            dm.get_skill_name_dict(),
        );

        dm.set_file_anomalies(anomalies);

        let talismans = parse_talisman(
            &manifest_dir.join("../bench-data/case1/talisman.txt"),
            dm.get_skill_name_dict(),
        );

        dm.set_file_talismans(talismans);

        let weapon_slots: Vec<SkillSlotCount> = serde_json::from_reader(BufReader::new(
            File::open("../bench-data/case1/weapon_slots.json").unwrap(),
        ))
        .unwrap();
        let selected_skills: HashMap<String, SkillSlotCount> = serde_json::from_reader(
            BufReader::new(File::open("../bench-data/case1/selected_skills.json").unwrap()),
        )
        .unwrap();
        let selected_skills: IntMap<usize, SkillSlotCount> = selected_skills
            .iter()
            .map(|(skill_id, level)| (dm.get_skill_uid(skill_id), *level))
            .collect();
        let free_slots: Vec<SkillSlotCount> = serde_json::from_reader(BufReader::new(
            File::open("../bench-data/case1/free_slots.json").unwrap(),
        ))
        .unwrap();

        let mut cm = CalcDataManager::new(&dm);
        cm.load_base_armors(&dm);
        cm.load_anomalies(&dm);
        cm.load_talismans(&dm);
        cm.refresh_infos(&dm, &convert_to_skills_container(&selected_skills));

        b.iter(move || {
            let (_log, _result) = calculate_skillset(
                weapon_slots.clone(),
                selected_skills.clone(),
                free_slots.clone(),
                SexType::Female,
                false,
                &dm,
                &cm,
            );
        });
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
