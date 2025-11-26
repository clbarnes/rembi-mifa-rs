//! REMBI model Rust representation
//!
//! This crate provides a set of structs to represent the REMBI model reference
//! (https://www.ebi.ac.uk/bioimage-archive/rembi-model-reference/) using
//! `serde` for (de)serialization and `validator` for basic field validation.

pub use super::mifa::{AnnotationType, FileLevelMetadata};
pub use iref::UriBuf;
pub use jiff::Zoned;
use serde::{Deserialize, Serialize};
use url::Url;
use validator::{Validate, ValidationErrors};

use super::{Doi, OrcId};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Affiliation {
    Url(OrganisationUrl),
    Info(OrganisationInfo),
}

impl Affiliation {
    pub fn new_url(name: String, url: Url) -> Self {
        Self::Url(OrganisationUrl { name, url })
    }

    pub fn new_info(name: String, address: String) -> Self {
        Self::Info(OrganisationInfo { name, address })
    }
}

impl Validate for Affiliation {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            Affiliation::Url(organisation_url) => organisation_url.validate(),
            Affiliation::Info(organisation_info) => organisation_info.validate(),
        }
    }
}

/// A person contributing to a study or annotation.
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Author {
    pub last_name: String,

    pub first_name: String,

    #[validate(email)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Normalised to URL form with hyphen separators when serialised.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orcid: Option<OrcId>,

    #[validate(nested)]
    pub affiliation: Affiliation,

    /// Author role in the study.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

impl Author {
    pub fn new(first_name: String, last_name: String, affiliation: Affiliation) -> Self {
        Self {
            first_name,
            last_name,
            affiliation,
            email: None,
            orcid: None,
            role: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct OrganisationUrl {
    #[validate(length(min = 1))]
    pub name: String,
    /// URL to a public registry containing organisation information. ROR
    /// recommended.
    pub url: Url,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct OrganisationInfo {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct GrantReference {
    pub identifier: String,

    pub funder: String,
}

impl GrantReference {
    pub fn new(identifier: String, funder: String) -> Self {
        Self { identifier, funder }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Funding {
    pub funding_statement: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub grant_references: Vec<GrantReference>,
}

impl Funding {
    pub fn new(funding_statement: String) -> Self {
        Self {
            funding_statement,
            grant_references: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Publication {
    #[validate(length(min = 1))]
    pub title: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub authors: Vec<Author>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate()]
    pub doi: Option<Doi>,

    /// Implementation note: the specification states that this is a FreeText field,
    /// implying that it is to be serialised as a string.
    /// So that is what we do.
    #[serde(skip_serializing_if = "Option::is_none", with = "super::u16_as_str")]
    pub year: Option<u16>,

    // probably some constraints in here...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubmed_id: Option<String>,
}

impl Publication {
    pub fn new(title: String) -> Self {
        Self {
            title,
            authors: Default::default(),
            doi: Default::default(),
            year: Default::default(),
            pubmed_id: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Link {
    /// A link URL (e.g., external resource).
    pub link_url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_description: Option<String>,
}

impl Link {
    pub fn new(link_url: Url) -> Self {
        Self {
            link_url,
            link_type: Default::default(),
            link_description: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct StudyComponent {
    pub name: String,

    pub description: String,

    pub rembi_version: monostate::MustBe!("1.5"),
}

impl StudyComponent {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            rembi_version: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Organism {
    pub scientific_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub common_name: Option<String>,

    // probably some constraints (e.g. URL)
    #[validate(length(min = 1))]
    pub ncbi_taxon: String,
}

impl Organism {
    pub fn new(scientific_name: String, ncbi_taxon: String) -> Self {
        Self {
            scientific_name,
            common_name: Default::default(),
            ncbi_taxon,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Biosample {
    #[validate(nested)]
    pub organism: Organism,

    pub biological_entity: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Intrinsic (e.g. genetic) alteration.
    ///
    /// Distinction between omitted/null "no variables recorded"
    /// and empty array "no explicit variables" (e.g. control)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intrinsic_variables: Option<Vec<String>>,

    /// External treatment (e.g. reagent).
    ///
    /// Distinction between omitted/null "no variables recorded"
    /// and empty array "no explicit variables" (e.g. control)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extrinsic_variables: Option<Vec<String>>,

    /// What is intentionally varied between multiple images.
    ///
    /// Implementation note: Distinction between omitted/null "no variables recorded"
    /// and empty array "no explicit variables" (e.g. control)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_variables: Option<Vec<String>>,
}

impl Biosample {
    pub fn new(organism: Organism, biological_entity: String) -> Self {
        Self {
            organism,
            biological_entity,
            description: Default::default(),
            intrinsic_variables: Default::default(),
            extrinsic_variables: Default::default(),
            experimental_variables: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Specimen {
    /// How the sample was prepared for imaging.
    pub sample_preparation: String,

    /// How the specimen was grown, e.g. cell line cultures, crosses or plant growth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub growth_protocol: Option<String>,
}

impl Specimen {
    pub fn new(sample_preparation: String) -> Self {
        Self {
            sample_preparation,
            growth_protocol: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct ImagingMethod {
    /// The text description of the ontology entry.
    pub value: String,

    pub ontology_name: String,

    /// The URI identifier for the ontology value.
    pub ontology_id: UriBuf,
}

impl ImagingMethod {
    pub fn new(value: String, ontology_name: String, ontology_id: UriBuf) -> Self {
        Self {
            value,
            ontology_name,
            ontology_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct ImageAcquisition {
    #[validate(nested)]
    pub imaging_method: ImagingMethod,

    /// Description of the instrument used to capture the images.
    pub imaging_instrument: String,

    /// How the images were acquired, including instrument settings/parameters
    pub image_acquisition_parameters: String,
}

impl ImageAcquisition {
    pub fn new(
        imaging_method: ImagingMethod,
        imaging_instrument: String,
        image_acquisition_parameters: String,
    ) -> Self {
        Self {
            imaging_method,
            imaging_instrument,
            image_acquisition_parameters,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct ImageCorrelation {
    /// Method used to correlate images from different modalities.
    pub spatial_and_temporal_alignment: String,

    /// Features from correlated datasets used for colocalisation.
    pub fiducials_used: String,

    /// Correlation transforms.
    ///
    /// Implementation note: this probably wants to be something more descriptive than a string.
    pub transformation_matrix: String,
}

impl ImageCorrelation {
    pub fn new(
        spatial_and_temporal_alignment: String,
        fiducials_used: String,
        transformation_matrix: String,
    ) -> Self {
        Self {
            spatial_and_temporal_alignment,
            fiducials_used,
            transformation_matrix,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct ImageAnalysis {
    /// How image analysis was carried out.
    pub analysis_overview: String,
}

impl ImageAnalysis {
    pub fn new(analysis_overview: String) -> Self {
        Self { analysis_overview }
    }
}

/// Implementation note: this probably needs fields but is empty in the spec.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct License;

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct Study {
    /// The title for your dataset. This will be displayed when search results including your data are shown. Often this will be the same as an associated publication.
    #[validate(length(min = 25))]
    pub title: String,

    /// Use this field to describe your dataset. This can be the abstract to an accompanying publication.
    #[validate(length(min = 25))]
    pub description: String,

    /// Date until which the study is private.
    pub private_until_date: jiff::civil::Date,

    /// Keywords describing your data that can be used to aid search and classification.
    ///
    /// Implementation notes: the specification does not require a particular delimiter.
    #[serde(default)]
    pub keywords: String,

    /// Implementation notes: the specification does not require that the vec is non-empty.
    #[validate(nested)]
    pub authors: Vec<Author>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub funding: Option<Funding>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub publications: Vec<Publication>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub links: Vec<Link>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub acknowledgements: Option<String>,

    pub rembi_version: monostate::MustBe!("1.5"),
}

impl Study {
    pub fn new(
        title: String,
        description: String,
        private_until_date: jiff::civil::Date,
        keywords: String,
        authors: Vec<Author>,
    ) -> Self {
        Self {
            title,
            description,
            private_until_date,
            keywords,
            authors,
            license: Default::default(),
            funding: Default::default(),
            publications: Default::default(),
            links: Default::default(),
            acknowledgements: Default::default(),
            rembi_version: Default::default(),
        }
    }
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

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct RembiStudy {
    #[validate(nested)]
    pub study: Study,

    #[validate(nested)]
    pub study_components: Vec<StudyComponent>,

    #[validate(nested)]
    pub sample: Vec<Biosample>,

    #[validate(nested)]
    pub specimen: Vec<Specimen>,

    #[validate(nested)]
    pub image_acquisition: Vec<ImageAcquisition>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub image_correlation: Option<ImageCorrelation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub image_analysis: Option<ImageAnalysis>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub annotations: Option<Annotations>,
}

impl RembiStudy {
    pub fn new(
        study: Study,
        study_components: Vec<StudyComponent>,
        sample: Vec<Biosample>,
        specimen: Vec<Specimen>,
        image_acquisition: Vec<ImageAcquisition>,
    ) -> Self {
        Self {
            study,
            study_components,
            sample,
            specimen,
            image_acquisition,
            image_correlation: Default::default(),
            image_analysis: Default::default(),
            annotations: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn basic_study_validation_and_serialization() {
        let author = Author {
            last_name: "Smith".into(),
            first_name: "Jane".into(),
            email: Some("jane.smith@example.org".into()),
            orcid: Default::default(),
            affiliation: Affiliation::Info(OrganisationInfo {
                name: "myorg".into(),
                address: Default::default(),
            }),
            role: Some("Corresponding author".into()),
        };

        let study = Study {
            title: "Example REMBI study".into(),
            description: "A minimal example of a REMBI Study struct".into(),
            private_until_date: jiff::civil::Date::ZERO,
            keywords: "example, rembi".into(),
            authors: vec![author.clone()],
            license: Default::default(),
            funding: Default::default(),
            publications: Default::default(),
            links: Default::default(),
            acknowledgements: Default::default(),
            rembi_version: Default::default(),
        };

        let rs = RembiStudy {
            study,
            study_components: Default::default(),
            sample: Default::default(),
            specimen: Default::default(),
            image_acquisition: Default::default(),
            image_correlation: Default::default(),
            image_analysis: Default::default(),
            annotations: Default::default(),
        };

        // Validate
        assert!(rs.validate().is_ok());

        // Serialize to JSON and back
        let json = serde_json::to_string_pretty(&rs).expect("serialize");
        let parsed: RembiStudy = serde_json::from_str(&json).expect("deserialize");
        assert!(parsed.validate().is_ok());
    }
}
