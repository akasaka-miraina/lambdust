# エフェクトシステムガイド

Lambdustは、組み合わせ可能で型安全な副作用とモナディックプログラミングを提供する洗練された代数エフェクトシステムを実装しています。このガイドでは、完全なエフェクトシステムの実装と高度な使用パターンについて説明します。

## 概要

Lambdustエフェクトシステムは以下を提供します：

- **代数エフェクト**: ハンドラーベースの解釈による組み合わせ可能エフェクト
- **エフェクトハンドラー**: 異なる実行コンテキストのためのプラガブルな解釈
- **モナディックプログラミング**: 完全なモナドとモナド変換子のサポート
- **エフェクト協調**: 並行エフェクト実行のための高度な協調機能
- **エフェクト分離**: 安全なエフェクト実行のためのサンドボックスとリソース管理
- **世代別エフェクト**: エフェクトライフサイクルとメモリ管理の統合

## 核となる概念

### 代数操作としてのエフェクト

エフェクトはシグネチャを持つ代数操作として定義されます：

```scheme
;; エフェクト型の定義
(define-effect-type Console
  ;; シグネチャを持つ操作
  (print : String -> Unit)
  (read-line : Unit -> String)
  (print-error : String -> Unit))

;; 状態を持つエフェクトの定義
(define-effect-type (State s)
  (get : Unit -> s)
  (put : s -> Unit)
  (modify : (s -> s) -> Unit))

;; リソースエフェクトの定義
(define-effect-type Resource
  (acquire : String -> Handle)
  (release : Handle -> Unit)
  (with-resource : String (Handle -> a) -> a))
```

### エフェクトハンドラー

ハンドラーはエフェクトに対する具体的な解釈を提供します：

```scheme
;; 標準I/Oを使用するコンソールハンドラー
(define console-io-handler
  (handler Console
    [(print msg) 
     (display msg)
     (newline)]
    [(read-line)
     (read-line (current-input-port))]
    [(print-error msg)
     (display msg (current-error-port))
     (newline (current-error-port))]))

;; テスト用コンソールハンドラー（出力を収集）
(define (console-test-handler)
  (let ([output-buffer '()]
        [input-queue '()])
    (handler Console
      [(print msg)
       (set! output-buffer (cons msg output-buffer))]
      [(read-line)
       (if (null? input-queue)
           (error "テストで入力が利用できません")
           (let ([input (car input-queue)])
             (set! input-queue (cdr input-queue))
             input))]
      [(print-error msg)
       (set! output-buffer (cons (string-append "エラー: " msg) output-buffer))]
      ;; 収集した出力へのアクセスを提供
      [get-output (reverse output-buffer)]
      [set-input! (lambda (inputs) (set! input-queue inputs))])))
```

## 基本エフェクト使用法

### シンプルなエフェクトプログラム

```scheme
;; 基本的なエフェクト計算
(define (greet-user)
  (do [_ (print "お名前は何ですか？")]
      [name (read-line)]
      [_ (print (string-append "こんにちは、" name "さん！"))]
      (return name)))

;; 特定のハンドラーで実行
(with-handler console-io-handler
  (greet-user))

;; 同じプログラムをテスト
(let ([test-handler (console-test-handler)])
  (test-handler 'set-input! '("太郎"))
  (with-handler test-handler
    (greet-user))
  ;; 収集された出力をチェック
  (assert (equal? (test-handler 'get-output)
                  '("お名前は何ですか？" "こんにちは、太郎さん！"))))
```

### 状態を持つ計算

```scheme
;; Stateエフェクトを使用したカウンター
(define (increment-counter)
  (do [current (get)]
      [_ (put (+ current 1))]
      (return current)))

(define (counter-program)
  (do [a (increment-counter)]
      [b (increment-counter)]  
      [c (increment-counter)]
      (return (list a b c))))

;; ステートハンドラー
(define (state-handler initial-state)
  (let ([state initial-state])
    (handler (State Integer)
      [(get) state]
      [(put new-state) (set! state new-state)]
      [(modify f) (set! state (f state))]
      ;; 最終状態へのアクセスを許可
      [final-state state])))

;; 使用法
(let ([handler (state-handler 0)])
  (with-handler handler
    (let ([result (counter-program)])
      (display (format "結果: ~a" result))          ;; (0 1 2)  
      (display (format "最終状態: ~a" (handler 'final-state)))))) ;; 3
```

## 高度なエフェクトパターン

### エフェクト合成

```scheme
;; 複数のエフェクトを組み合わせ
(define (logging-counter name)
  (do [_ (print (string-append "カウンターを開始: " name))]
      [initial (get)]
      [_ (print (format "初期値: ~a" initial))]
      [result (increment-counter)]
      [_ (print (format "~aに増加しました" (+ initial 1)))]
      (return result)))

;; 複数のハンドラーを使用
(with-handler console-io-handler
  (with-handler (state-handler 10)
    (logging-counter "テストカウンター")))
```

### カスタムエフェクト定義

```scheme
;; ファイルシステムエフェクト
(define-effect-type FileSystem
  (read-file : String -> String)
  (write-file : String String -> Unit)
  (file-exists? : String -> Boolean)
  (delete-file : String -> Unit))

;; 実際のファイルシステムハンドラー
(define filesystem-handler
  (handler FileSystem
    [(read-file filename)
     (call-with-input-file filename
       (lambda (port) (read-string #f port)))]
    [(write-file filename content)
     (call-with-output-file filename
       (lambda (port) (display content port)))]
    [(file-exists? filename)
     (file-exists? filename)]
    [(delete-file filename)
     (delete-file filename)]))

;; テスト用インメモリファイルシステム
(define (memory-filesystem-handler)
  (let ([files (make-hash-table)])
    (handler FileSystem
      [(read-file filename)
       (hash-table-ref files filename 
         (lambda () (error "ファイルが見つかりません" filename)))]
      [(write-file filename content)
       (hash-table-set! files filename content)]
      [(file-exists? filename)
       (hash-table-exists? files filename)]
      [(delete-file filename)
       (hash-table-delete! files filename)]
      ;; テスト用ユーティリティ
      [list-files (hash-table-keys files)]
      [clear! (hash-table-clear! files)])))
```

### エフェクトを使ったエラーハンドリング

```scheme
;; 例外エフェクト
(define-effect-type (Exception e)
  (throw : e -> a)
  (catch : (Unit -> a) (e -> a) -> a))

;; 例外ハンドラー
(define (exception-handler)
  (handler (Exception String)
    [(throw error) (error error)]
    [(catch computation handler)
     (guard (condition
             [(string? condition) (handler condition)])
       (computation))]))

;; 安全なファイル操作
(define (safe-read-file filename)
  (catch
    (lambda () (read-file filename))
    (lambda (error) 
      (print-error (format "~aの読み込みに失敗: ~a" filename error))
      "")))

;; 複数エフェクトでの使用
(define (process-config-file)
  (do [config-content (safe-read-file "config.json")]
      [_ (if (string-null? config-content)
             (print "デフォルト設定を使用します")
             (print "カスタム設定を読み込みました"))]
      [config (parse-json config-content)]
      (return config)))
```

## モナディックプログラミング

### 組み込みモナド

```scheme
;; nullable値用Maybeモナド
(define-monad Maybe
  (return : a -> (Maybe a))
  (bind : (Maybe a) (a -> (Maybe b)) -> (Maybe b)))

(define-type (Maybe a)
  (Nothing)
  (Just a))

(define-instance (Monad Maybe)
  (define (return x) (Just x))
  (define (bind maybe f)
    (match maybe
      [(Nothing) (Nothing)]
      [(Just x) (f x)])))

;; 非決定的計算用Listモナド
(define-instance (Monad List)
  (define (return x) (list x))
  (define (bind lst f)
    (concatenate (map f lst))))

;; 使用例
(define (safe-divide x y)
  (if (= y 0)
      (Nothing)
      (Just (/ x y))))

(define (computation)
  (do [x (Just 10)]
      [y (Just 2)] 
      [result (safe-divide x y)]
      (return (* result 2))))
;; 結果: (Just 10)

;; 非決定的計算
(define (pythagorean-triples n)
  (do [x (range 1 n)]
      [y (range x n)]
      [z (range y n)]
      (if (= (+ (* x x) (* y y)) (* z z))
          (return (list x y z))
          (list))))  ;; 空のリスト = 失敗
```

### モナド変換子

```scheme
;; StateTモナド変換子
(define-monad-transformer StateT
  (lift-monad : (m a) -> (StateT s m a))
  (run-state : (StateT s m a) s -> (m (Pair a s))))

;; MaybeT変換子
(define-monad-transformer MaybeT
  (lift-monad : (m a) -> (MaybeT m a))
  (run-maybe : (MaybeT m a) -> (m (Maybe a))))

;; 変換子の組み合わせ
(define-type (StateMaybeT s m a)
  (StateMaybeT (StateT s (MaybeT m) a)))

;; 例: ログ付きで失敗する可能性のある状態計算
(define (stateful-division x y)
  (do [_ (lift-io (print (format "~aを~aで割ります" x y)))]
      [current-state (get)]
      [_ (put (+ current-state 1))]  ;; 操作をカウント
      (if (= y 0)
          (lift-maybe (Nothing))     ;; 失敗
          (return (/ x y)))))        ;; 成功

;; 変換子スタックの実行
(define (run-computation)
  (let ([result (run-state 
                  (run-maybe
                    (run-io
                      (stateful-division 10 2)))
                  0)])
    result))
```

## エフェクト協調

### 並行エフェクト

```scheme
;; 並列エフェクト実行
(define (parallel-file-processing filenames)
  (parallel-map
    (lambda (filename)
      (do [content (read-file filename)]
          [processed (process-content content)]
          [output-name (string-append filename ".processed")]
          [_ (write-file output-name processed)]
          (return output-name)))
    filenames))

;; エフェクト同期
(define (coordinated-updates)
  (let ([barrier (make-barrier 3)])
    (parallel-eval
      (spawn (do [data1 (read-file "data1.txt")]
                 [_ (barrier-wait barrier)]
                 [_ (process-data data1)]
                 (return "タスク1完了")))
      (spawn (do [data2 (read-file "data2.txt")]
                 [_ (barrier-wait barrier)]  
                 [_ (process-data data2)]
                 (return "タスク2完了")))
      (spawn (do [config (read-file "config.json")]
                 [_ (barrier-wait barrier)]
                 [_ (apply-config config)]
                 (return "タスク3完了"))))))
```

### エフェクト分離

```scheme
;; サンドボックス化されたエフェクト実行
(define (run-untrusted-code code)
  (with-effect-sandbox
    (sandbox-config
      (allow-effects [Console])           ;; コンソールI/Oのみ許可
      (deny-effects [FileSystem Network]) ;; ファイルとネットワークアクセスをブロック
      (resource-limits
        (memory-limit 10MB)
        (time-limit 30s)
        (cpu-limit 50%)))
    (eval-with-effects code)))

;; リソース管理されたエフェクト
(define (with-database-transaction f)
  (with-resource "database-connection"
    (lambda (conn)
      (do [_ (begin-transaction conn)]
          [result (guard (condition
                          [else 
                           (rollback-transaction conn)
                           (throw condition)])
                    (f conn))]
          [_ (commit-transaction conn)]
          (return result)))))
```

## パフォーマンスと最適化

### エフェクトコンパイル

```scheme
;; パフォーマンス向上のためのエフェクト融合
#:optimize-effects #t

(define (pipeline data)
  (do [step1 (map-effect transform1 data)]    ;; これらのエフェクトは
      [step2 (map-effect transform2 step1)]   ;; 単一の操作に融合可能
      [step3 (filter-effect predicate step2)]
      (return step3)))

;; 効率的な単一パス操作にコンパイルされる
```

### エフェクト特殊化

```scheme
;; ホットパス用の特殊化ハンドラー
(define fast-console-handler
  (handler Console
    [(print msg)
     ;; パフォーマンス重要パス用に特殊化
     (fast-display msg)]    ;; 直接システムコール
    [(read-line)
     (fast-read-line)]))    ;; 最適化入力

;; 純粋計算のエフェクトインライン化
(define (pure-computation x)
  #:inline-effects #t
  (return (* x x)))        ;; エフェクトメカニズムが除去される
```

## 型システムとの統合

### エフェクト型

```scheme
;; エフェクトを持つ関数型
(define (read-config) : (FileSystem String)
  (read-file "config.json"))

(define (log-message msg : String) : (Console Unit)
  (print (string-append "[ログ] " msg)))

(define (interactive-setup) : (Console FileSystem (Config))
  (do [_ (log-message "セットアップを開始しています")]
      [name (do [_ (print "名前を入力してください: ")]
                (read-line))]
      [config (make-config name)]
      [_ (write-file "user.config" (serialize config))]
      (return config)))
```

### エフェクト推論

```scheme
;; エフェクトは使用から推論される
(define (process-user-data username)    ;; 推論: FileSystem Console IO
  (do [user-file (string-append username ".json")]
      [_ (print (format "~aを処理しています" username))]     ;; Consoleエフェクト
      [data (read-file user-file)]                     ;; FileSystemエフェクト  
      [processed (http-get processing-service data)]    ;; IOエフェクト
      [_ (write-file (string-append username ".processed") processed)]
      (return processed)))
```

## 高度な例

### 完全なエフェクトシステム使用例

```scheme
#!/usr/bin/env lambdust

(import (scheme base)
        (lambdust effects)
        (lambdust monads)
        (lambdust concurrency))

;; アプリケーション固有エフェクトの定義
(define-effect-type Database
  (query : String -> (List Record))
  (insert : Record -> Id)
  (update : Id Record -> Unit)
  (delete : Id -> Unit))

(define-effect-type Logging  
  (debug : String -> Unit)
  (info : String -> Unit)
  (warn : String -> Unit)
  (error : String -> Unit))

(define-effect-type WebServer
  (handle-request : Request -> Response)
  (start-server : Number -> Unit)
  (stop-server : Unit -> Unit))

;; コネクションプールを持つデータベースハンドラー
(define (database-handler connection-pool)
  (handler Database
    [(query sql)
     (with-connection connection-pool
       (lambda (conn) (execute-query conn sql)))]
    [(insert record)
     (with-connection connection-pool
       (lambda (conn) (execute-insert conn record)))]
    [(update id record)
     (with-connection connection-pool
       (lambda (conn) (execute-update conn id record)))]
    [(delete id)
     (with-connection connection-pool
       (lambda (conn) (execute-delete conn id)))]))

;; 構造化ログハンドラー
(define (logging-handler level)
  (handler Logging
    [(debug msg) (when (<= level 0) (log-output "デバッグ" msg))]
    [(info msg)  (when (<= level 1) (log-output "情報" msg))]
    [(warn msg)  (when (<= level 2) (log-output "警告" msg))]
    [(error msg) (when (<= level 3) (log-output "エラー" msg))]))

;; エフェクトを持つWebアプリケーション
(define (user-service)
  (define (get-user id)
    (do [_ (debug (format "ユーザー~aを取得中" id))]
        [users (query (format "SELECT * FROM users WHERE id = ~a" id))]
        (if (null? users)
            (do [_ (warn (format "ユーザー~aが見つかりません" id))]
                (return (http-response 404 "ユーザーが見つかりません")))
            (do [_ (info (format "ユーザー~aを発見" id))]
                (return (http-response 200 (serialize (car users))))))))

  (define (create-user user-data)
    (do [_ (info "新しいユーザーを作成中")]
        [user (deserialize user-data)]
        [user-id (insert user)]
        [_ (info (format "ID ~aでユーザーを作成しました" user-id))]
        (return (http-response 201 (format "{\"id\": ~a}" user-id)))))

  ;; ルートハンドラー
  (lambda (request)
    (match (request-path request)
      [(string-append "/users/" id)
       (if (equal? (request-method request) "GET")
           (get-user (string->number id))
           (http-response 405 "メソッドが許可されていません"))]
      ["/users"
       (if (equal? (request-method request) "POST")
           (create-user (request-body request))
           (http-response 405 "メソッドが許可されていません"))]
      [_
       (http-response 404 "見つかりません")])))

;; メインアプリケーション
(define (run-app)
  (let ([db-pool (create-connection-pool "postgresql://localhost/myapp")]
        [port 8080])
    (with-handler (database-handler db-pool)
      (with-handler (logging-handler 1) ;; 情報レベル以上
        (with-handler web-server-handler
          (do [_ (info (format "ポート~aでサーバーを開始" port))]
              [service (user-service)]
              [_ (handle-request service)]  ;; ハンドラーを登録
              [_ (start-server port)]
              [_ (info "サーバーが正常に開始されました")]
              ;; サーバーを動作させ続ける
              (server-loop)))))))

;; グレースフルシャットダウン
(define (shutdown-handler signal)
  (do [_ (info "シャットダウンシグナルを受信")]
      [_ (stop-server)]
      [_ (info "サーバーが停止しました")]
      (exit 0)))

;; アプリケーションエントリーポイント
(when (script-file?)
  (install-signal-handler 'SIGTERM shutdown-handler)
  (install-signal-handler 'SIGINT shutdown-handler)
  (run-app))
```

### エフェクトシステムテスト

```scheme
;; 包括的エフェクトテストフレームワーク
(define (test-user-service)
  ;; モックデータベース
  (let ([test-db (make-hash-table)]
        [next-id 1])
    (define mock-db-handler
      (handler Database
        [(query sql) (hash-table-values test-db)]
        [(insert record)
         (let ([id next-id])
           (set! next-id (+ next-id 1))
           (hash-table-set! test-db id record)
           id)]
        [(update id record) (hash-table-set! test-db id record)]
        [(delete id) (hash-table-delete! test-db id)]))
    
    ;; モックログ（メッセージを収集）
    (let ([log-messages '()])
      (define mock-logging-handler
        (handler Logging
          [(debug msg) (set! log-messages (cons (cons 'debug msg) log-messages))]
          [(info msg)  (set! log-messages (cons (cons 'info msg) log-messages))]
          [(warn msg)  (set! log-messages (cons (cons 'warn msg) log-messages))]
          [(error msg) (set! log-messages (cons (cons 'error msg) log-messages))]
          [get-messages (reverse log-messages)]
          [clear-messages! (set! log-messages '())]))
      
      ;; テスト実行
      (with-handler mock-db-handler
        (with-handler mock-logging-handler
          (let ([service (user-service)])
            ;; ユーザー作成テスト
            (let ([response (service (make-request "POST" "/users" "{\"name\": \"太郎\"}"))])
              (assert (= (response-status response) 201))
              (assert (member '(info . "新しいユーザーを作成中") 
                             (mock-logging-handler 'get-messages))))
            
            ;; ユーザー取得テスト
            (let ([response (service (make-request "GET" "/users/1" ""))])
              (assert (= (response-status response) 200))
              (assert (string-contains? (response-body response) "太郎")))
            
            ;; ユーザー未発見テスト
            (let ([response (service (make-request "GET" "/users/999" ""))])
              (assert (= (response-status response) 404))
              (assert (member '(warn . "ユーザー999が見つかりません")
                             (mock-logging-handler 'get-messages))))))))))

;; テスト実行
(test-user-service)
(display "すべてのテストが合格しました！")
```

このエフェクトシステムは、関数プログラミングパラダイムを保持し、洗練されたプログラム解析と最適化を可能にしながら、堅牢でテスト可能で保守しやすいアプリケーションを構築するための強力な基盤を提供します。