use tachikoma_common_fs as fs;
use tachikoma_common_core::Result;

#[test]
fn test_integration_with_core_types() {
    // Test that errors integrate properly with core error types
    let result: Result<String> = fs::read_to_string("/nonexistent/file", 1024);
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert_eq!(error.category(), tachikoma_common_core::ErrorCategory::FileSystem);
    assert_eq!(error.code(), tachikoma_common_core::ErrorCode::FILE_NOT_FOUND);
}