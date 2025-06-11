use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Read, Write},
    process::exit,
    time::Duration,
};

use crate::types::{
    Agreement, ArticulationContainer, AvailableMajors, Course, Institution, ResultContainer, Year,
};
use indicatif::ProgressIterator;
use inquire::Select;
use itertools::Itertools;
use platform_dirs::AppDirs;

mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_dirs = AppDirs::new(Some("reverCCCe"), true).unwrap();
    let cache_dir = app_dirs.cache_dir;
    std::fs::create_dir(&cache_dir);

    let institutions = reqwest::get("https://assist.org/api/institutions")
        .await?
        .json::<Vec<Institution>>()
        .await?;

    let mut options = institutions
        .iter()
        .filter(|i| !i.is_community_college)
        .collect::<Vec<_>>();

    let my_school = Select::new("Which university are you attending?", options).prompt()?;

    let mut agreements = reqwest::get(format!(
        "https://assist.org/api/institutions/{}/agreements",
        my_school.id
    ))
    .await?
    .json::<Vec<Agreement>>()
    .await?;

    agreements.retain(|ag| ag.is_community_college);

    println!("Downloading agreements");

    let mut majors = HashMap::new();

    for ag in agreements.iter().progress() {
        if ag.sending_year_ids.is_empty() {
            continue;
        }
        let mut was_cached = false;
        let urlstring = format!(
            "https://assist.org/api/agreements?receivingInstitutionId={}&sendingInstitutionId={}&academicYearId={}&categoryCode=major",
            my_school.id,
            ag.institution_parent_id,
            ag.sending_year_ids.last().unwrap()
        );

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(cache_dir.join(format!(
                "Ag-{}-{}-{}.json",
                my_school.id,
                ag.institution_parent_id,
                ag.sending_year_ids.last().unwrap()
            )));

        let agreement = match file {
            Ok(mut f) => {
                let mut buf = String::new();
                f.read_to_string(&mut buf)?;
                if buf.is_empty() {
                    let req = reqwest::get(urlstring).await?;
                    let text = req.text().await?;
                    let v = serde_json::from_str::<AvailableMajors>(&text)?;
                    f.write_all(text.as_bytes())?;
                    v
                } else {
                    was_cached = true;

                    serde_json::from_str::<AvailableMajors>(&buf)?
                }
            }
            Err(e) => {
                let req = reqwest::get(urlstring).await?;
                let text = req.text().await?;
                serde_json::from_str::<AvailableMajors>(&text)?
            }
        };

        for major in agreement.reports {
            let major_label = major.label;
            if majors.contains_key(&major_label) {
                let list: &mut Vec<(i64, String, String)> = majors.get_mut(&major_label).unwrap();
                list.push((
                    ag.institution_parent_id,
                    ag.institution_name.clone(),
                    major.key,
                ));
                list.dedup();
            } else {
                majors.insert(
                    major_label,
                    vec![(
                        ag.institution_parent_id,
                        ag.institution_name.clone(),
                        major.key,
                    )],
                );
            }
        }
        if !was_cached {
            std::thread::sleep(Duration::from_secs_f64(8.0));
        }
    }

    let mut majors_names = majors.iter().map(|m| m.0.as_str()).collect::<Vec<_>>();
    majors_names.sort();

    let my_major = Select::new("What is your major?", majors_names).prompt()?;

    println!("Downloading articulations");

    let mut classes = HashMap::new();

    for art in majors.get(my_major).unwrap().iter().progress() {
        let urlstr = format!(
            "https://assist.org/api/articulation/Agreements?key={}",
            art.2
        );

        let mut was_cached = false;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(cache_dir.join(format!(
                "Art-{}.json",
                art.2.replace('/', "-").replace('\\', "-")
            )));

        let res_con = match file {
            Ok(mut f) => {
                let mut buf = String::new();
                f.read_to_string(&mut buf)?;
                if buf.is_empty() {
                    let req = reqwest::get(urlstr).await?;
                    let text = req.text().await?;
                    let v = serde_json::from_str::<ResultContainer>(&text)?;
                    f.write_all(text.as_bytes())?;
                    v
                } else {
                    was_cached = true;

                    serde_json::from_str::<ResultContainer>(&buf)?
                }
            }
            Err(e) => {
                let req = reqwest::get(urlstr).await?;
                let text = req.text().await?;
                serde_json::from_str::<ResultContainer>(&text)?
            }
        };

        //println!("{:#?}", &res_con);
        let articulations: Result<Vec<ArticulationContainer>, serde_json::Error> =
            serde_json::from_str::<Vec<ArticulationContainer>>(&res_con.result.articulations);
        if articulations.is_err() {
            eprintln!("{}", articulations.unwrap_err());
            eprintln!("{:#?}", res_con.result.articulations);
            panic!()
        }
        let articulations = articulations.unwrap();

        for class in articulations {
            let mut uni_class_name = String::new();
            match class.articulation.type_field.as_str() {
                "Course" => {
                    uni_class_name = class.articulation.course.unwrap().course_title;
                }
                "Series" => {
                    uni_class_name = class.articulation.series.clone().unwrap().name.clone();
                }
                "Requirement" => {
                    uni_class_name = class.articulation.requirement.unwrap().name;
                }
                a @ _ => {
                    eprintln!("Unhandled articulation type \"{a}\"");
                }
            };

            let result_string = {
                let group_conj = class
                    .articulation
                    .sending_articulation
                    .course_group_conjunctions
                    .as_slice();
                let mut course_groups = class.articulation.sending_articulation.items.clone();
                course_groups.sort_by(|a, b| a.position.cmp(&b.position));
                let course_grp_strings = course_groups
                    .iter()
                    .map(|cg| {
                        let conj = format!(" {} ", cg.course_conjunction);
                        let mut courses = cg.items.clone();
                        let n_courses = courses.len();
                        courses.sort_by(|a, b| a.position.cmp(&b.position));
                        let mut st = courses.iter().map(|c| c.course_title.clone()).join(&conj);
                        if n_courses > 1 {
                            st = format!("[{}]", st);
                        }
                        st
                    })
                    .collect_vec();
                if course_grp_strings.len() == 1 {
                    course_grp_strings[0].clone()
                } else {
                    group_conj
                        .iter()
                        .map(|gc| {
                            let n = (gc.sending_course_group_begin_position
                                ..=gc.sending_course_group_end_position)
                                .count();
                            let mut s = (gc.sending_course_group_begin_position
                                ..=gc.sending_course_group_end_position)
                                .map(|idx| course_grp_strings[idx as usize].clone())
                                .join(format!(" {} ", gc.group_conjunction).as_str());
                            if n > 1 {
                                s = format!("({})", s);
                            }
                            s
                        })
                        .join(", ")
                }
            };
            if classes.contains_key(&uni_class_name) {
                let list: &mut Vec<(String, String)> = classes.get_mut(&uni_class_name).unwrap();
                list.push((art.1.clone(), result_string));
                list.dedup();
            } else {
                classes.insert(uni_class_name, vec![(art.1.clone(), result_string)]);
            }
        }
        if !was_cached {
            std::thread::sleep(Duration::from_secs_f64(8.0));
        }
    }
    let classes_names = classes.iter().map(|c| c.0.clone()).collect_vec();

    if classes_names.is_empty() {
        println!("No available articulations");
    } else {
        let my_class = Select::new("Select a class", classes_names).prompt()?;
        let classes = classes.get(&my_class).unwrap();
        let uni_max_w = classes.iter().map(|x| x.0.len()).max().unwrap_or(0) + 1;
        for class in classes.iter().map(|(a, b)| (format!("{a}:"), b)) {
            println!("{:w$}  {}", class.0, class.1, w = uni_max_w);
        }
    }

    std::io::stdin().read_line(&mut String::new()).unwrap();

    Ok(())
}
