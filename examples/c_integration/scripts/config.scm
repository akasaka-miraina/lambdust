;; Configuration script for config_example

;; Application metadata
(app-config "Lambdust Demo Application" "1.5.2")

;; UI Configuration
(window-size 1200 800)

;; Performance settings
(network 150 30.0)

;; Development vs Production settings
(let ((environment (env-or-default "NODE_ENV" "development")))
  (cond 
    ((string=? environment "production")
     (logging "ERROR" #f)
     (data-dir "/var/lib/myapp"))
    ((string=? environment "staging") 
     (logging "WARN" #f)
     (data-dir "/tmp/myapp-staging"))
    (else ; development
     (logging "DEBUG" #t)
     (data-dir "./dev-data"))))

;; Plugin configuration
(plugins "authentication" 
         "database" 
         "file-manager" 
         "notification-system")

;; Feature flags based on configuration
(when-file-exists "enable-experimental.flag"
  (lambda ()
    (add-plugin! "experimental-features")
    (set-config! "debug-enabled" "true")))

;; Memory-based configuration adjustments
(let ((available-memory (env-or-default "AVAILABLE_MEMORY" "1024")))
  (if (< (string->number available-memory) 512)
      (begin
        (set-config! "max-connections" "20")
        (add-plugin! "memory-optimizer"))
      (begin
        (set-config! "max-connections" "100")
        (add-plugin! "high-performance-mode"))))

;; Completion message
(display "Application configuration completed successfully")
(newline)