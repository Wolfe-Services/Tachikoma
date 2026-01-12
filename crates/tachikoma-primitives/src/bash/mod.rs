    #[tokio::test]
    async fn test_bash_output_truncation() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new()
            .max_stdout(10) // Very small limit
            .max_stderr(10);
        
        // Command that outputs more than 10 bytes
        let result = bash(&ctx, "echo 'This is a very long output line that should be truncated'", Some(opts)).await.unwrap();
        
        assert!(result.is_output_truncated());
        assert!(result.stdout_total_bytes > 10);
        assert_eq!(result.stdout.len(), 10);
    }

    #[tokio::test]
    async fn test_bash_separate_output_capture() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = bash(&ctx, "echo 'stdout message' && echo 'stderr message' >&2", None).await.unwrap();
        
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("stdout message"));
        assert!(result.stderr.contains("stderr message"));
        assert!(!result.is_output_truncated());
    }

    #[tokio::test]
    async fn test_bash_combined_output() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let result = bash(&ctx, "echo 'stdout' && echo 'stderr' >&2", None).await.unwrap();
        
        let combined = result.combined_output();
        assert!(combined.contains("stdout"));
        assert!(combined.contains("stderr"));
    }

    #[tokio::test]
    async fn test_bash_ansi_stripping() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new().strip_ansi(true);
        let result = bash(&ctx, "echo -e '\\x1b[31mRed\\x1b[0m Text'", Some(opts)).await.unwrap();
        
        // ANSI codes should be stripped
        assert!(result.stdout.contains("Red Text"));
        assert!(!result.stdout.contains("\x1b"));
    }

    #[tokio::test]
    async fn test_bash_no_ansi_stripping() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new().strip_ansi(false);
        let result = bash(&ctx, "echo -e '\\x1b[31mRed\\x1b[0m Text'", Some(opts)).await.unwrap();
        
        // ANSI codes should remain
        assert!(result.stdout.contains("\x1b"));
    }

    #[tokio::test] 
    async fn test_bash_output_trimming() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new().trim_output(true);
        let result = bash(&ctx, "echo '  padded output  '", Some(opts)).await.unwrap();
        
        // Output should be trimmed
        assert_eq!(result.stdout, "padded output");
    }

    #[tokio::test]
    async fn test_bash_no_output_trimming() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        let opts = BashOptions::new().trim_output(false);
        let result = bash(&ctx, "echo '  padded output  '", Some(opts)).await.unwrap();
        
        // Output should include whitespace
        assert!(result.stdout.starts_with("  "));
        assert!(result.stdout.ends_with("  \n"));
    }

    #[tokio::test]
    async fn test_bash_binary_output_handling() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        // Create some binary data (should not panic)
        let result = bash(&ctx, "printf '\\x00\\x01\\x02\\xFF'", None).await.unwrap();
        
        assert_eq!(result.exit_code, 0);
        // Binary data should be handled without panic using lossy UTF-8 conversion
        assert!(result.stdout.len() > 0);
    }