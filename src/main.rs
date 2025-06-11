use std::{collections::HashMap, process::exit, time::Duration};

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

    for ag in agreements.iter().take(20).progress() {
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

    for art in majors.get(my_major).unwrap().iter().take(20).progress() {
        let res_con = reqwest::get(format!("https://assist.org/api/articulation/Agreements?key={}", art.2))
            .await?
            .json::<ResultContainer>().await?;
        
        //println!("{:#?}", &res_con);
        let articulations: Vec<ArticulationContainer> = serde_json::from_str(&res_con.result.articulations)?;
        
        for class in articulations {
            let mut uni_class_name = String::new();
            match class.articulation.type_field.as_str() {
                "Course" => {uni_class_name = class.articulation.course.unwrap().course_title;}
                "Series" => {uni_class_name = class.articulation.series.clone().unwrap().name.clone();
                }
                "Requirement" => {uni_class_name = class.articulation.requirement.unwrap().name;}
                a @ _ => {
                    eprintln!("Unhandled articulation type \"{a}\"");}
            };
            
            let result_string = {
                    let group_conj = class.articulation.sending_articulation.course_group_conjunctions.as_slice();
                    let mut course_groups = class.articulation.sending_articulation.items.clone();
                    course_groups.sort_by(|a,b|a.position.cmp(&b.position));
                    let course_grp_strings = course_groups.iter().map(|cg| {
                        let conj = format!(" {} ", cg.course_conjunction);
                        let mut courses = cg.items.clone();
                        let n_courses = courses.len();
                        courses.sort_by(|a,b| a.position.cmp(&b.position));
                        let mut st = courses.iter().map(|c|c.course_title.clone()).join(&conj);
                        if n_courses > 1 {
                            st = format!("[{}]", st);
                        }
                        st
                    }).collect_vec();
                    if course_grp_strings.len() == 1 {
                        course_grp_strings[0].clone()
                    }
                    else {
                        group_conj.iter().map(|gc|{
                            let n = (gc.sending_course_group_begin_position..=gc.sending_course_group_end_position).count();
                            let mut s = (gc.sending_course_group_begin_position..=gc.sending_course_group_end_position).map(|idx| {
                                course_grp_strings[idx as usize].clone()
                            }).join(format!(" {} ", gc.group_conjunction).as_str());
                            if n > 1 {
                                s = format!("({})", s);
                            }
                            s
                        }).join(", ")
                    }

                
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
    let classes = classes.get(&my_class).unwrap();
    let uni_max_w = classes.iter().map(|x|x.0.len()).max().unwrap_or(0) + 1;
    for class in classes.iter().map(|(a,b)| (format!("{a}:"), b)) {

        
        println!("{:w$}  {}", class.0, class.1, w=uni_max_w);
    }

    std::io::stdin().read_line(&mut String::new()).unwrap();

    Ok(())
}
