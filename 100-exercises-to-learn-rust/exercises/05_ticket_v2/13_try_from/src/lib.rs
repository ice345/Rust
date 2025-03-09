// TODO: Implement `TryFrom<String>` and `TryFrom<&str>` for `Status`.
//  The parsing should be case-insensitive.

#[derive(Debug, PartialEq, Clone)]
enum Status {
    ToDo,
    InProgress,
    Done,
}

#[derive(Debug, thiserror::Error)]
#[error("{invalid_status} is not a valid status")]
struct ParseStatusError {
    invalid_status: String,
}

impl TryFrom<String> for Status {
    type Error = ParseStatusError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into() //这个其实是将其转化为&str直接调用了impl的&str代码
    }
}

impl TryFrom<&str> for Status {
    type Error = ParseStatusError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "todo" => Ok(Self::ToDo),
            "inprogress" => Ok(Self::InProgress),
            "done" => Ok(Self::Done),
            _ => Err(ParseStatusError {
                invalid_status: value.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_try_from_string() {
        let status = Status::try_from("ToDO".to_string()).unwrap();
        assert_eq!(status, Status::ToDo);

        let status = Status::try_from("inproGress".to_string()).unwrap();
        assert_eq!(status, Status::InProgress);

        let status = Status::try_from("Done".to_string()).unwrap();
        assert_eq!(status, Status::Done);
    }

    #[test]
    fn test_try_from_str() {
        let status = Status::try_from("todo").unwrap();
        assert_eq!(status, Status::ToDo);

        let status = Status::try_from("inprogress").unwrap();
        assert_eq!(status, Status::InProgress);

        let status = Status::try_from("done").unwrap();
        assert_eq!(status, Status::Done);
    }
}
