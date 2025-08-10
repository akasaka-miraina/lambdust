# 並行性ガイド

Lambdustは、アクターシステム、並列評価、高度な同期プリミティブを組み合わせた包括的な並行性モデルを提供します。このガイドでは、完全な並行性インフラストラクチャと使用パターンについて説明します。

## 並行性概要

Lambdustの並行性システムには以下が含まれます：

- **アクターモデル**: 分離された状態を持つメッセージパッシング並行性
- **並列評価**: 計算の自動並列化
- **高度な同期**: パフォーマンス最適化を伴うスレッドセーフプリミティブ
- **ソフトウェア・トランザクショナル・メモリ**: 組み合わせ可能なアトミック操作
- **分散コンピューティング**: ネットワーク透過的なアクター通信
- **エフェクト認識並行性**: 代数エフェクトシステムとの統合

## アクターモデル

### 基本アクターシステム

```rust
use lambdust::concurrency::actors::{Actor, ActorSystem, Message};

// アクターの定義
#[derive(Debug)]
struct CounterActor {
    count: i64,
    name: String,
}

#[derive(Debug, Clone)]
enum CounterMessage {
    Increment(i64),
    Decrement(i64),
    GetCount(tokio::sync::oneshot::Sender<i64>),
    Reset,
}

impl Actor for CounterActor {
    type Message = CounterMessage;
    
    async fn handle(&mut self, message: Self::Message) -> Result<(), Box<dyn std::error::Error>> {
        match message {
            CounterMessage::Increment(delta) => {
                self.count += delta;
                println!("[{}] {}だけ増加、カウント: {}", self.name, delta, self.count);
            }
            CounterMessage::Decrement(delta) => {
                self.count -= delta;
                println!("[{}] {}だけ減少、カウント: {}", self.name, delta, self.count);
            }
            CounterMessage::GetCount(sender) => {
                let _ = sender.send(self.count);
            }
            CounterMessage::Reset => {
                println!("[{}] {}から0にリセット", self.name, self.count);
                self.count = 0;
            }
        }
        Ok(())
    }
}

// アクターシステムの使用
async fn actor_example() -> Result<(), Box<dyn std::error::Error>> {
    let system = ActorSystem::new();
    
    // アクターをスポーン
    let counter1 = system.spawn(CounterActor { count: 0, name: "カウンター1".to_string() }).await?;
    let counter2 = system.spawn(CounterActor { count: 100, name: "カウンター2".to_string() }).await?;
    
    // メッセージ送信
    counter1.send(CounterMessage::Increment(5)).await?;
    counter2.send(CounterMessage::Decrement(10)).await?;
    
    // 状態の照会
    let (tx, rx) = tokio::sync::oneshot::channel();
    counter1.send(CounterMessage::GetCount(tx)).await?;
    let count = rx.await?;
    println!("カウンター1の最終カウント: {}", count);
    
    Ok(())
}
```

### Schemeアクターインターフェース

```scheme
;; Schemeでアクターを定義
(define-actor bank-account
  (state (balance 0)
         (account-id "")
         (transaction-log '()))
  
  ;; 入金メッセージの処理
  (handle (deposit amount)
    (when (> amount 0)
      (set! balance (+ balance amount))
      (set! transaction-log 
            (cons (make-transaction 'deposit amount (current-time))
                  transaction-log))
      (log-info (format "~a円を入金、新残高: ~a円" amount balance)))
    balance)
  
  ;; 出金メッセージの処理
  (handle (withdraw amount)
    (cond [(> amount balance)
           (log-warn (format "残高不足: ~a円 > ~a円" amount balance))
           (error 'insufficient-funds)]
          [(> amount 0)
           (set! balance (- balance amount))
           (set! transaction-log
                 (cons (make-transaction 'withdraw amount (current-time))
                       transaction-log))
           (log-info (format "~a円を出金、新残高: ~a円" amount balance))
           balance]
          [else
           (error 'invalid-amount)]))
  
  ;; 残高照会の処理
  (handle (get-balance)
    balance)
  
  ;; 取引履歴の処理
  (handle (get-transactions)
    (reverse transaction-log)))

;; アクターの使用
(define account (spawn-actor bank-account 
                             #:initial-state '((account-id "ACC001"))))

;; アクターにメッセージ送信
(send-message account (deposit 100000))     ;; 戻り値: 100000
(send-message account (withdraw 25000))     ;; 戻り値: 75000
(send-message account (get-balance))        ;; 戻り値: 75000
```

### アクター監視

```scheme
;; フォルトトレラント用スーパーバイザーアクター
(define-supervisor-actor banking-supervisor
  (supervision-strategy 'one-for-one)  ;; 失敗した子のみ再起動
  (max-restarts 5)
  (restart-window 60) ;; 60秒間で最大5回の再起動
  
  (supervise account-actors
    (lambda (child-spec)
      (spawn-actor bank-account 
                   #:name (child-spec-name child-spec)
                   #:initial-state (child-spec-state child-spec))))
  
  ;; 子の失敗処理
  (on-child-failure (child-ref reason)
    (log-error (format "アカウントアクター~aが失敗: ~a" 
                      (actor-name child-ref) reason))
    ;; カスタム再起動ロジック
    (restart-child child-ref)))

;; 監視されたアクターシステムの作成
(define supervisor (spawn-supervisor banking-supervisor))
(define account1 (supervisor-spawn-child supervisor 
                                        'account-1 
                                        '((account-id "ACC001"))))
```

## 並列評価

### 自動並列化

```scheme
;; 並列map - 作業を自動分散
(define (parallel-processing data)
  (parallel-map 
    (lambda (item)
      (expensive-computation item))
    data))

;; カスタム結合器を持つ並列fold
(define (parallel-sum numbers)
  (parallel-fold + 0 numbers))

;; 並列フィルター
(define (parallel-find-primes numbers)
  (parallel-filter prime? numbers))

;; 並列度レベルの制御
(parameterize ([max-parallelism 4])
  (parallel-map computation large-dataset))
```

### フューチャとプロミス

```scheme
;; 非同期計算用フューチャの作成
(define future1 (future (expensive-computation-1)))
(define future2 (future (expensive-computation-2)))
(define future3 (future (expensive-computation-3)))

;; フューチャ結果の組み合わせ
(define combined-result
  (future
    (+ (force future1)
       (force future2)
       (force future3))))

;; プロミスベースの協調
(define promise1 (make-promise))
(define promise2 (make-promise))

;; プロデューサースレッド
(spawn
  (lambda ()
    (let ([result (compute-data)])
      (deliver-promise promise1 result))))

;; コンシューマースレッド
(spawn
  (lambda ()
    (let ([data (promise-value promise1)])
      (let ([processed (process-data data)])
        (deliver-promise promise2 processed)))))

;; 最終結果の待機
(define final-result (promise-value promise2))
```

### ワークスティーリング並列性

```rust
use lambdust::concurrency::parallel::{WorkStealingPool, Task};

// ワークスティーリングスレッドプールの作成
let pool = WorkStealingPool::new()
    .with_thread_count(num_cpus::get())
    .with_queue_size(10000)
    .with_work_stealing_strategy(WorkStealingStrategy::Randomized);

// 並列タスクの提出
let tasks: Vec<Task<i32>> = data.iter().map(|item| {
    pool.submit(move || expensive_computation(*item))
}).collect();

// 準備完了時の結果収集
let results: Vec<i32> = tasks.into_iter()
    .map(|task| task.join().unwrap())
    .collect();
```

## 同期プリミティブ

### スレッドセーフデータ構造

```rust
use lambdust::concurrency::sync::{
    Mutex, RwLock, Semaphore, Barrier, 
    AtomicRef, LockFreeQueue, AtomicCounter
};

// 排他アクセス用ミューテックス
let shared_data = Mutex::new(HashMap::new());
{
    let mut guard = shared_data.lock().await;
    guard.insert("key".to_string(), "value".to_string());
}

// 複数リーダー、単一ライター用RwLock
let config = RwLock::new(ApplicationConfig::default());
{
    let read_guard = config.read().await;
    println!("現在の設定: {}", read_guard.get_setting("key"));
}
{
    let mut write_guard = config.write().await;
    write_guard.update_setting("key", "new_value");
}

// リソース制御用セマフォ
let resource_pool = Semaphore::new(5); // 5つの同時ユーザーを許可
let permit = resource_pool.acquire().await?;
// リソースを使用
drop(permit); // リソースを解放

// 同期用バリア
let barrier = Barrier::new(4); // 4つのスレッドを待機
barrier.wait().await;

// ロックフリーデータ構造
let queue = LockFreeQueue::new();
queue.push(item).await?;
let item = queue.pop().await?;

// アトミック操作
let counter = AtomicCounter::new(0);
let old_value = counter.fetch_add(1, Ordering::SeqCst);
```

### Scheme同期インターフェース

```scheme
;; Schemeでのミューテックス使用
(define shared-counter (make-mutex 0))

(define (increment-shared-counter)
  (with-mutex shared-counter
    (lambda (current)
      (+ current 1))))

;; 複数スレッドが安全に増加
(parallel-eval
  (spawn (lambda () (for-each (lambda (_) (increment-shared-counter)) 
                             (range 0 1000))))
  (spawn (lambda () (for-each (lambda (_) (increment-shared-counter)) 
                             (range 0 1000))))
  (spawn (lambda () (for-each (lambda (_) (increment-shared-counter)) 
                             (range 0 1000)))))

;; 最終結果は3000であるべき
(display (format "最終カウンター値: ~a" (mutex-value shared-counter)))

;; 協調用条件変数
(define buffer (make-bounded-buffer 10))
(define buffer-not-empty (make-condition-variable))
(define buffer-not-full (make-condition-variable))

;; プロデューサー
(define (producer items)
  (for-each
    (lambda (item)
      (with-mutex buffer
        (lambda ()
          (when (buffer-full? buffer)
            (condition-wait buffer-not-full))
          (buffer-put! buffer item)
          (condition-signal buffer-not-empty))))
    items))

;; コンシューマー
(define (consumer)
  (with-mutex buffer
    (lambda ()
      (when (buffer-empty? buffer)
        (condition-wait buffer-not-empty))
      (let ([item (buffer-get! buffer)])
        (condition-signal buffer-not-full)
        item))))
```

## ソフトウェア・トランザクショナル・メモリ

### STM基本

```scheme
;; トランザクショナル変数の定義
(define account1-balance (make-tvar 100000))
(define account2-balance (make-tvar 50000))
(define transaction-log (make-tvar '()))

;; アトミック送金
(define (transfer from-account to-account amount)
  (atomic
    (let ([from-balance (tvar-read from-account)]
          [to-balance (tvar-read to-account)])
      (if (>= from-balance amount)
          (begin
            (tvar-write! from-account (- from-balance amount))
            (tvar-write! to-account (+ to-balance amount))
            (tvar-write! transaction-log 
                        (cons (make-transaction from-account to-account amount)
                              (tvar-read transaction-log)))
            #t) ;; 成功
          #f)))) ;; 残高不足

;; 並行送金 - すべてアトミック
(parallel-eval
  (spawn (lambda () (transfer account1-balance account2-balance 20000)))
  (spawn (lambda () (transfer account2-balance account1-balance 10000)))
  (spawn (lambda () (transfer account1-balance account2-balance 5000))))

;; 並行性にも関わらず残高は一貫性を保つ
(display (format "口座1: ~a円、口座2: ~a円" 
                (tvar-read account1-balance)
                (tvar-read account2-balance)))
```

### 組み合わせ可能なトランザクション

```scheme
;; 組み合わせ可能なトランザクショナル操作
(define (withdraw account amount)
  (atomic
    (let ([balance (tvar-read account)])
      (if (>= balance amount)
          (begin
            (tvar-write! account (- balance amount))
            amount)
          (retry))))) ;; 残高が十分になるまで再試行

(define (deposit account amount)
  (atomic
    (let ([balance (tvar-read account)])
      (tvar-write! account (+ balance amount))
      (+ balance amount))))

;; トランザクションの組み合わせ
(define (transfer-with-fee from to amount fee-account fee)
  (atomic
    (withdraw from amount)
    (deposit to (- amount fee))
    (deposit fee-account fee)))

;; orElseを使った代替組み合わせ
(define (try-transfer-from-either account1 account2 target amount)
  (or-else
    (atomic (transfer account1 target amount))
    (atomic (transfer account2 target amount))))
```

## 分散並行性

### リモートアクター

```scheme
;; 分散アクターシステム
(define-distributed-actor distributed-counter
  (state (count 0)
         (node-id (get-node-id)))
  
  (handle (increment)
    (set! count (+ count 1))
    (broadcast-to-replicas `(sync-count ,count ,node-id))
    count)
  
  (handle (sync-count remote-count remote-node)
    (when (> remote-count count)
      (log-info (format "ノード~aからカウントを同期: ~a" remote-node remote-count))
      (set! count remote-count)))
  
  (handle (get-count)
    count))

;; 複数ノードへの配置
(define node1-counter (spawn-distributed-actor 
                        distributed-counter 
                        #:node "node1.cluster.local"))
(define node2-counter (spawn-distributed-actor 
                        distributed-counter 
                        #:node "node2.cluster.local"))

;; ネットワーク越しでも操作は透過的に動作
(send-message node1-counter (increment))  ;; node1で更新
(send-message node2-counter (get-count))  ;; node2から読み取り（同期済み）
```

### ネットワーク透過通信

```rust
use lambdust::concurrency::distributed::{DistributedNode, ClusterConfig};

// 分散コンピューティングクラスターのセットアップ
let cluster_config = ClusterConfig {
    node_id: "worker-1".to_string(),
    cluster_peers: vec![
        "worker-2.cluster.local:9000".to_string(),
        "worker-3.cluster.local:9000".to_string(),
    ],
    heartbeat_interval: Duration::from_secs(5),
    failure_detection_timeout: Duration::from_secs(15),
};

let node = DistributedNode::new(cluster_config).await?;

// 分散計算のスポーン
let distributed_task = node.spawn_distributed_task(
    "compute-pi",
    |start: i64, end: i64| {
        // モンテカルロ法を使ってπを計算
        monte_carlo_pi_segment(start, end)
    },
    vec![(0, 1000000), (1000000, 2000000), (2000000, 3000000)]
).await?;

// 全ノードから結果収集
let results = distributed_task.collect().await?;
let pi_estimate = results.iter().sum::<f64>() / 3.0;
```

## エフェクト認識並行性

### 並行エフェクト

```scheme
;; 並行コンテキストでのエフェクト
(define (concurrent-file-processing filenames)
  (with-effect-coordination
    (parallel-map
      (lambda (filename)
        (do [content (read-file filename)]           ;; FileSystemエフェクト
            [_ (log-info (format "~aを処理中" filename))] ;; Loggingエフェクト
            [processed (process-content content)]
            [output (string-append filename ".processed")]
            [_ (write-file output processed)]
            (return output)))
      filenames)))

;; アクター間のエフェクト分離
(define-actor file-processor
  (with-effects [FileSystem Logging])  ;; 許可されたエフェクトを宣言
  
  (handle (process-file filename)
    (with-effect-isolation
      (do [content (read-file filename)]
          [result (transform-content content)]
          [_ (log-info (format "~aを処理: ~aバイト" 
                              filename (string-length result)))]
          (return result)))))
```

### 協調エフェクトハンドリング

```rust
use lambdust::runtime::effect_coordination::{
    EffectCoordinator, ConcurrentEffectSystem, EffectIsolationLevel
};

// 並行システム用のエフェクト協調セットアップ
let coordinator = EffectCoordinator::new()
    .with_concurrent_effects(true)
    .with_isolation_level(EffectIsolationLevel::Strict)
    .with_resource_limits(ResourceLimits {
        max_concurrent_effects: 100,
        memory_limit: 512 * 1024 * 1024, // 512MB
        time_limit: Duration::from_secs(60),
    });

// 協調を伴う複数アクター間でのエフェクト実行
let concurrent_system = ConcurrentEffectSystem::new(coordinator);
let results = concurrent_system.execute_parallel_effects(
    vec![
        Effect::FileOperation(FileOp::Read("file1.txt".to_string())),
        Effect::FileOperation(FileOp::Read("file2.txt".to_string())),
        Effect::NetworkOperation(NetworkOp::HttpGet("api.example.com".to_string())),
    ]
).await?;
```

## パフォーマンス最適化

### ロックフリープログラミング

```rust
use lambdust::concurrency::sync::{AtomicRef, LockFreeQueue, AtomicCounter};
use std::sync::atomic::{AtomicPtr, Ordering};

// ロックフリースタック実装
struct LockFreeStack<T> {
    head: AtomicPtr<Node<T>>,
}

struct Node<T> {
    data: T,
    next: *mut Node<T>,
}

impl<T> LockFreeStack<T> {
    fn new() -> Self {
        Self {
            head: AtomicPtr::new(std::ptr::null_mut()),
        }
    }
    
    fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data,
            next: std::ptr::null_mut(),
        }));
        
        loop {
            let head = self.head.load(Ordering::Acquire);
            unsafe { (*new_node).next = head; }
            
            if self.head.compare_exchange_weak(
                head,
                new_node,
                Ordering::Release,
                Ordering::Relaxed
            ).is_ok() {
                break;
            }
        }
    }
    
    fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            if head.is_null() {
                return None;
            }
            
            let next = unsafe { (*head).next };
            if self.head.compare_exchange_weak(
                head,
                next,
                Ordering::Release,
                Ordering::Relaxed
            ).is_ok() {
                let data = unsafe { Box::from_raw(head).data };
                return Some(data);
            }
        }
    }
}
```

### 高性能アクターメッセージング

```scheme
;; 高スループットメッセージング用最適化アクター
(define-actor high-throughput-processor
  (state (processed-count 0)
         (batch-buffer '())
         (batch-size 1000))
  
  ;; より良いスループットのためのバッチ処理
  (handle (process-item item)
    (set! batch-buffer (cons item batch-buffer))
    (when (>= (length batch-buffer) batch-size)
      (let ([batch (reverse batch-buffer)])
        (set! batch-buffer '())
        (set! processed-count (+ processed-count batch-size))
        (process-batch batch))))
  
  ;; バックプレッシャーの処理
  (handle (get-queue-size)
    (actor-mailbox-size (self)))
  
  (on-mailbox-full (message sender)
    ;; 送信者を遅延させることでバックプレッシャーを適用
    (send-message sender (slow-down 100))))  ;; 100ms遅延

;; パフォーマンス監視付きで使用
(define processor (spawn-actor high-throughput-processor 
                              #:mailbox-size 10000
                              #:priority 'high))

;; パフォーマンス監視
(define (monitor-performance actor)
  (let loop ()
    (let ([queue-size (send-message-sync actor (get-queue-size))])
      (when (> queue-size 5000)  ;; 高いキューサイズ
        (log-warn "高いメッセージキューサイズを検出"))
      (thread-sleep 1000)  ;; 毎秒チェック
      (loop))))
```

## デバッグと監視

### 並行性デバッグ

```scheme
;; 並行アクターのデバッグ
(define-debug-actor debug-counter
  (enable-message-tracing #t)
  (enable-state-snapshots #t)
  
  (state (count 0))
  
  (handle (increment)
    (debug-trace "カウントを~aから増加中" count)
    (set! count (+ count 1))
    (debug-snapshot 'count count)
    count))

;; デッドロック検出
(with-deadlock-detection
  (with-mutex mutex1
    (lambda ()
      (with-mutex mutex2
        (lambda ()
          (critical-section))))))

;; 競合状態検出
(define (test-race-conditions)
  (with-race-detection
    (let ([shared-var 0])
      (parallel-eval
        (spawn (lambda () (set! shared-var (+ shared-var 1))))
        (spawn (lambda () (set! shared-var (+ shared-var 1)))))
      shared-var)))
```

### パフォーマンス監視

```rust
use lambdust::concurrency::monitoring::{ConcurrencyMonitor, ActorMetrics};

// アクターシステムパフォーマンスの監視
let monitor = ConcurrencyMonitor::new()
    .with_metrics([
        ActorMetrics::MessageThroughput,
        ActorMetrics::MailboxUtilization,
        ActorMetrics::ProcessingLatency,
        ActorMetrics::ErrorRates,
    ])
    .with_sampling_interval(Duration::from_secs(1));

monitor.start().await?;

// パフォーマンスメトリクスの照会
let metrics = monitor.snapshot().await?;
println!("メッセージ/秒: {}", metrics.message_throughput);
println!("平均メールボックス利用率: {:.1}%", metrics.avg_mailbox_utilization * 100.0);
println!("P99レイテンシ: {:?}", metrics.p99_latency);
```

## 設定

### 並行性システム設定

```toml
[concurrency]
# アクターシステム
actor_system_threads = "auto"  # CPUコア数
mailbox_default_size = 1000
supervisor_restart_strategy = "one_for_one"
max_restarts = 5
restart_window = "60s"

# 並列評価
parallel_threads = "auto"
work_stealing = true
task_queue_size = 10000
parallel_threshold = 100  # 並列化のための最小アイテム数

# 同期
mutex_spin_count = 1000
rwlock_prefer_writers = false
semaphore_fairness = true

# STM設定
stm_retry_limit = 1000
stm_contention_backoff = "exponential"
stm_gc_threshold = 10000

# 分散コンピューティング
cluster_heartbeat = "5s"
failure_timeout = "15s"
network_buffer_size = "64KB"
compression = "lz4"

# 監視
enable_metrics = true
metrics_sampling = "1s"
deadlock_detection = true  # 開発環境で有効
race_detection = false     # 高価、テスト用
```

この並行性ガイドは、効率的でスケーラブルなアプリケーションを構築するための高レベル抽象化と低レベル制御の両方を提供する、Lambdustの並行・並列プログラミングへの包括的アプローチを示しています。