use serde::{Deserialize, Serialize};
use url::Url;
use validator::Validate;

pub use super::{Doi, OrcId};

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct MifaContainer {
    #[validate(nested)]
    pub publications: Publications,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub authors: Vec<Author>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub grants: Vec<GrantReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub link_url: Vec<Url>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub link_description: Vec<String>,
    pub title: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    pub license: LicenseType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ai_models_trained: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acknowledgements: Option<String>,
    pub funding_statement: String,
    pub annotations: Vec<Annotations>,
}

impl MifaContainer {
    pub fn new(
        publications: Publications,
        title: String,
        description: String,
        license: LicenseType,
        funding_statement: String,
        annotations: Vec<Annotations>,
    ) -> Self {
        Self {
            publications,
            authors: Default::default(),
            grants: Default::default(),
            link_url: Default::default(),
            link_description: Default::default(),
            title,
            description,
            keywords: Default::default(),
            license,
            ai_models_trained: Default::default(),
            acknowledgements: Default::default(),
            funding_statement,
            annotations,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Publications {
    pub publication_title: String,
    pub publication_authors: String,
    pub publication_doi: Doi,
    #[serde(skip_serializing_if = "Option::is_none", with = "super::u16_as_str")]
    pub publication_year: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubmed_id: Option<String>,
}

impl Publications {
    pub fn new(
        publication_title: String,
        publication_authors: String,
        publication_doi: Doi,
    ) -> Self {
        Self {
            publication_title,
            publication_authors,
            publication_doi,
            publication_year: Default::default(),
            pubmed_id: Default::default(),
        }
    }
}

/// Information about the authors
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Author {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    organisation: Vec<OrganisationInfo>,
    author_first_name: String,
    author_last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(email)]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    orcid_id: Option<OrcId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    role: Vec<String>,
}

impl Author {
    pub fn new(author_first_name: String, author_last_name: String) -> Self {
        Self {
            organisation: Default::default(),
            author_first_name,
            author_last_name,
            email: Default::default(),
            orcid_id: Default::default(),
            role: Default::default(),
        }
    }
}

/// Information about the organisation the author is affiliated with
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OrganisationInfo {
    organisation_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ror_id: Option<String>,
}

impl OrganisationInfo {
    pub fn new(organisation_name: String) -> Self {
        Self {
            organisation_name,
            address: Default::default(),
            ror_id: Default::default(),
        }
    }
}

/// Information about grant ID and funding body that funded the study
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GrantReference {
    grant_id: String,
    funder: String,
}

impl GrantReference {
    pub fn new(grant_id: String, funder: String) -> Self {
        Self { grant_id, funder }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LicenseType {
    #[serde(rename = "CC0")]
    /// No Copyright. You can copy, modify, distribute and perform the work, even for commercial purposes, all without asking permission.
    Cc0,
    /// You are free to: Share — copy and redistribute the material in any medium or format. Adapt — remix, transform, and build upon the material for any purpose, even commercially. You must give appropriate credit, provide a link to the license, and indicate if changes were made. You may do so in any reasonable manner, but not in any way that suggests the licensor endorses you or your use.
    #[serde(rename = "CC_BY")]
    CcBy,
}

/// A set of annotations for an AI-ready dataset.
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Annotations {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub authors: Vec<Author>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub file_metadata: Vec<FileLevelMetadata>,

    pub annotation_overview: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotation_type: Vec<AnnotationType>,

    pub annotation_method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_criteria: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_coverage: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_confidence_level: Option<String>,
}

impl Annotations {
    pub fn new(annotation_overview: String, annotation_method: String) -> Self {
        Self {
            authors: Default::default(),
            file_metadata: Default::default(),
            annotation_overview,
            annotation_type: Default::default(),
            annotation_method,
            annotation_criteria: Default::default(),
            annotation_coverage: Default::default(),
            annotation_confidence_level: Default::default(),
        }
    }
}

/// Implementation note: this type is referred to in both REMBI and MIFA specifications,
/// but only defined in the MIFA specification.
/// Here we re-exported it in both modules.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationType {
    /// tags that identify specific features, patterns or classes in images
    ClassLabels,
    BoundingBoxes,
    Counts,
    DerivedAnnotations,
    GeometricalAnnotations,
    Graphs,
    PointAnnotations,
    SegmentationMask,
    Tracks,
    WeakAnnotations,
    /// Other types of annotations, please specify in the annotation overview section.
    Other,
}

/// This type is defined identically in both the REMBI and MIFA specifications.
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct FileLevelMetadata {
    pub annotation_id: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotation_type: Vec<AnnotationType>,

    pub source_image_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformations: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub spatial_information: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_creation_time: Option<jiff::Zoned>,
}

impl FileLevelMetadata {
    pub fn new(annotation_id: String, source_image_id: String) -> Self {
        Self {
            annotation_id,
            annotation_type: Default::default(),
            source_image_id,
            transformations: Default::default(),
            spatial_information: Default::default(),
            annotation_creation_time: Default::default(),
        }
    }
}
