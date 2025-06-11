use std::fmt::{Display};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Institution {
    pub id: i64,
    pub names: Vec<Name>,
    pub code: String,
    #[serde(rename = "prefers2016LegacyReport")]
    pub prefers2016legacy_report: bool,
    pub is_community_college: bool,
    pub category: i64,
    pub term_type: i64,
    pub begin_id: i64,
    pub term_type_academic_years: Vec<TermTypeAcademicYear>,
    pub end_id: Option<i64>,
}

impl Display for Institution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.names[0].name.fmt(f)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Name {
    pub name: String,
    pub has_departments: bool,
    pub hide_in_list: bool,
    pub from_year: Option<i64>,
    pub alternate_institution_id: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TermTypeAcademicYear {
    pub term_type: i64,
    pub from_year: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agreement {
    pub institution_parent_id: i64,
    pub institution_name: String,
    pub code: String,
    pub is_community_college: bool,
    pub sending_year_ids: Vec<i64>,
    pub receiving_year_ids: Vec<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Year {
    #[serde(rename = "Id")]
    pub id: i64,
    #[serde(rename = "FallYear")]
    pub fall_year: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableMajors {
    pub reports: Vec<Report>,
    pub all_reports: Vec<AllReport>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub label: String,
    pub key: String,
    pub owner_institution_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllReport {
    pub label: String,
    pub key: String,
    pub owner_institution_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultContainer {
    pub result: Result,
    pub validation_failure: Value,
    pub is_successful: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub publish_date: String,
    pub receiving_institution: String,
    pub sending_institution: String,
    pub academic_year: String,
    pub template_assets: String,
    pub articulations: String,
    pub catalog_year: String,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticulationContainer {
    pub template_cell_id: String,
    pub articulation: Articulation,
    pub receiving_attributes: ReceivingAttributes,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]

pub struct Articulation {
    #[serde(rename = "type")]
    pub type_field: String,
    pub course: Option<Course>,
    pub series: Option<Series>,
    #[serde(default)]
    pub visible_cross_listed_courses: Vec<Value>,
    #[serde(default)]
    pub course_attributes: Vec<Value>,
    pub sending_articulation: SendingArticulation,
    pub template_overrides: Vec<Value>,
    pub attributes: Vec<Value>,
    pub receiving_attributes: Vec<Value>,
    pub requirement: Option<Requirement>,
    #[serde(default)]
    pub requirement_attributes: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub position: i64,
    pub course_identifier_parent_id: i64,
    pub course_title: String,
    pub course_number: String,
    pub prefix: String,
    pub prefix_parent_id: i64,
    pub prefix_description: String,
    pub department_parent_id: i64,
    pub department: String,
    pub begin: String,
    pub end: String,
    pub min_units: f64,
    pub max_units: f64,
    pub pathways: Vec<Pathway>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub conjunction: String,
    pub name: String,
    pub courses: Vec<Course>,
    pub series_pathways: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pathway {
    pub pathway_name: String,
    pub pathway_id: i64,
    pub pathway_code: String,
    pub expectation_name: String,
    pub expectation_id: i64,
    pub subexpectation_name: Value,
    pub subexpectation_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendingArticulation {
    pub no_articulation_reason: Value,
    pub denied_courses: Vec<Value>,
    pub items: Vec<Item>,
    pub course_group_conjunctions: Vec<CourseGroupConjunction>,
    #[serde(rename = "type")]
    pub type_field: String,
    pub attributes: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub course_conjunction: String,
    pub items: Vec<Item2>,
    pub attributes: Vec<Value>,
    pub position: i64,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item2 {
    pub visible_cross_listed_courses: Vec<Value>,
    pub requisites: Vec<Value>,
    pub attributes: Vec<Value>,
    pub course_identifier_parent_id: i64,
    pub course_title: String,
    pub course_number: String,
    pub prefix: String,
    pub prefix_parent_id: i64,
    pub prefix_description: String,
    pub department_parent_id: i64,
    pub department: String,
    pub begin: String,
    pub end: String,
    pub min_units: f64,
    pub max_units: f64,
    pub pathways: Vec<Value>,
    pub published_course_identifier_year_term_id: Value,
    pub position: i64,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseGroupConjunction {
    pub id: String,
    pub sending_articulation_id: String,
    pub group_conjunction: String,
    pub sending_course_group_begin_position: i64,
    pub sending_course_group_end_position: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Requirement {
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivingAttributes {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub course_attributes: Vec<Value>,
    pub attributes: Vec<Value>,
    #[serde(default)]
    pub requirement_attributes: Vec<Value>,
}