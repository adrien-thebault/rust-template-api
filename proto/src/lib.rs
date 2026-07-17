pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor");

pub mod todo_service {
    tonic::include_proto!("{{crate_name}}.todo_service");
}

pub mod shared {
    tonic::include_proto!("{{crate_name}}.shared");

    impl From<SortDirection> for ::rust_toolbox::diesel_tools::SortDirection {
        fn from(value: SortDirection) -> Self {
            match value {
                SortDirection::Desc => Self::Desc,
                _ => Self::Asc,
            }
        }
    }

    impl From<SortField> for (String, ::rust_toolbox::diesel_tools::SortDirection) {
        fn from(field: SortField) -> Self {
            (
                field.field,
                SortDirection::try_from(field.direction)
                    .unwrap_or(SortDirection::Asc)
                    .into(),
            )
        }
    }

    impl From<PageRequest> for ::rust_toolbox::diesel_tools::PageRequest {
        fn from(request: PageRequest) -> Self {
            let sort = if request.sort.is_empty() {
                ::rust_toolbox::diesel_tools::Sort::Unsorted
            } else {
                ::rust_toolbox::diesel_tools::Sort::Sorted {
                    items: request.sort.into_iter().map(Into::into).collect(),
                }
            };

            match (request.offset, request.size) {
                (Some(offset), Some(size)) => Self::Paged { offset, size, sort },
                _ => Self::Unpaged { sort },
            }
        }
    }

    impl From<::rust_toolbox::diesel_tools::SortDirection> for SortDirection {
        fn from(value: ::rust_toolbox::diesel_tools::SortDirection) -> Self {
            match value {
                ::rust_toolbox::diesel_tools::SortDirection::Asc => Self::Asc,
                ::rust_toolbox::diesel_tools::SortDirection::Desc => Self::Desc,
            }
        }
    }

    impl From<(String, ::rust_toolbox::diesel_tools::SortDirection)> for SortField {
        fn from((field, direction): (String, ::rust_toolbox::diesel_tools::SortDirection)) -> Self {
            SortField {
                field,
                direction: SortDirection::from(direction) as i32,
            }
        }
    }

    impl From<::rust_toolbox::diesel_tools::PageRequest> for PageRequest {
        fn from(request: ::rust_toolbox::diesel_tools::PageRequest) -> Self {
            let sort_to_proto = |sort| match sort {
                ::rust_toolbox::diesel_tools::Sort::Sorted { items } => {
                    items.into_iter().map(Into::into).collect()
                }
                ::rust_toolbox::diesel_tools::Sort::Unsorted => vec![],
            };

            match request {
                ::rust_toolbox::diesel_tools::PageRequest::Paged { offset, size, sort } => Self {
                    offset: Some(offset),
                    size: Some(size),
                    sort: sort_to_proto(sort),
                },
                ::rust_toolbox::diesel_tools::PageRequest::Unpaged { sort } => Self {
                    offset: None,
                    size: None,
                    sort: sort_to_proto(sort),
                },
            }
        }
    }

    /// builds the response-side pagination metadata for a `Page<T>` - doesn't
    /// depend on `T`, since [`PageResponse`] never carries the actual items
    /// (see the message's own doc comment in `src/shared.proto`).
    impl<T> From<&::rust_toolbox::diesel_tools::Page<T>> for PageResponse {
        fn from(page: &::rust_toolbox::diesel_tools::Page<T>) -> Self {
            Self {
                total_elements: page.total_elements,
                page_request: Some(page.page_request.clone().into()),
                next_page: page.next_page().map(Into::into),
                previous_page: page.previous_page().map(Into::into),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn paged_and_sorted_request_converts() {
            let proto_request = PageRequest {
                offset: Some(10),
                size: Some(20),
                sort: vec![SortField {
                    field: "name".to_string(),
                    direction: SortDirection::Desc as i32,
                }],
            };

            let request: ::rust_toolbox::diesel_tools::PageRequest = proto_request.into();
            match request {
                ::rust_toolbox::diesel_tools::PageRequest::Paged { offset, size, sort } => {
                    assert_eq!(offset, 10);
                    assert_eq!(size, 20);
                    assert!(matches!(
                        sort,
                        ::rust_toolbox::diesel_tools::Sort::Sorted { items } if items == vec![("name".to_string(), ::rust_toolbox::diesel_tools::SortDirection::Desc)]
                    ));
                }
                other => panic!("expected a Paged request, got {other:?}"),
            }
        }
    }
}
