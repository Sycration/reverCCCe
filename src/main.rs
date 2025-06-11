use std::{collections::HashMap, time::Duration};

use crate::types::{Agreement, ArticulationContainer, AvailableMajors, Course, Institution, ResultContainer, Year};
use indicatif::ProgressIterator;
use inquire::Select;
use itertools::Itertools;

mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let years = reqwest::get("https://assist.org/api/academicyears")
    // .await?
    // .json::<Vec<Year>>().await?;

    // let current_year = years[0].id;

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

    agreements.retain(|ag|ag.is_community_college);

    println!("Downloading agreements");

    let mut majors = HashMap::new();

    for ag in agreements.iter().take(10).progress() {
        if ag.sending_year_ids.is_empty() {
            continue;
        }
        let urlstring = format!("https://assist.org/api/agreements?receivingInstitutionId={}&sendingInstitutionId={}&academicYearId={}&categoryCode=major", my_school.id, ag.institution_parent_id,ag.sending_year_ids.last().unwrap());
        // println!("{urlstring}");
        let agreement = reqwest::get(urlstring)
            .await?
            .json::<AvailableMajors>().await?;

        for major in agreement.reports {
            let major_label = major.label;
            if majors.contains_key(&major_label) {
                let list: &mut Vec<(i64, String, String)> = majors.get_mut(&major_label).unwrap();
                list.push((ag.institution_parent_id, ag.institution_name.clone(), major.key));
                list.dedup();
            }
            else {
                majors.insert(major_label, vec![(ag.institution_parent_id, ag.institution_name.clone(), major.key)]);
                
            }
        }
        std::thread::sleep(Duration::from_secs_f64(0.1));
    }

    let mut majors_names = majors.iter().map(|m|m.0.as_str()).collect::<Vec<_>>();
    majors_names.sort();

    let my_major = Select::new("What is your major?", majors_names).prompt()?;

    println!("Downloading articulations");

    let mut classes = HashMap::new();

    for art in majors.get(my_major).unwrap().iter().take(10).progress() {
        let res_con = reqwest::get(format!("https://assist.org/api/articulation/Agreements?key={}", art.2))
            .await?
            .json::<ResultContainer>().await?;
        
        //println!("{:#?}", &res_con);
        let articulations: Vec<ArticulationContainer> = serde_json::from_str(&res_con.result.articulations)?;
        
        for class in articulations {
            let mut uni_class_name = String::new();
            match class.articulation.type_field.as_str() {
                "Course" => {uni_class_name = class.articulation.course.unwrap().course_title;}
                _ => {continue;}
            }
            let result_string = match class.articulation.type_field.as_str() {
                "Course" => {
                    if class.articulation.sending_articulation.items.is_empty() {
                        continue;
                    }
                    class.articulation.sending_articulation.items[0].items[0].course_title.clone()
                },
                _ => {continue;}
            };
            if classes.contains_key(&uni_class_name) {
                let list: &mut Vec<(String, String)> = classes.get_mut(&uni_class_name).unwrap();
                list.push((art.1.clone(), result_string));
                list.dedup();
            } else {
                classes.insert(uni_class_name, vec![(
                    art.1.clone(),
                    result_string
                )]);
            }
        }

    }
    let classes_names = classes.iter().map(|c|c.0.clone()).collect_vec();

    let my_class = Select::new("Select a class", classes_names).prompt()?;
    for class in classes.get(&my_class).unwrap() {
        println!("{}:\t{}", class.0, class.1);
    }



    Ok(())
}
