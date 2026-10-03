use std::collections::HashMap;

pub enum BatchLabelError {
    Empty {
        index: usize,
    },
    Duplicate {
        label: String,
        first_index: usize,
        duplicate_index: usize,
    },
}

pub fn validate_labels<'a>(
    labels: impl IntoIterator<Item = &'a str>,
) -> Result<(), BatchLabelError> {
    let mut seen = HashMap::new();
    for (index, label) in labels.into_iter().enumerate() {
        if label.is_empty() {
            return Err(BatchLabelError::Empty { index });
        }
        if let Some(first_index) = seen.insert(label, index) {
            return Err(BatchLabelError::Duplicate {
                label: label.to_owned(),
                first_index,
                duplicate_index: index,
            });
        }
    }
    Ok(())
}
