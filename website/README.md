# Lambdust Official Website - Simple Version

純粋なHTML+CSS+JavaScriptで構築されたLambdustの公式ウェブサイト。

## 🎯 設計方針

### 軽量性重視
- **バンドルサイズ**: HTML+CSS+JS合計 < 50KB
- **依存関係**: ゼロ（純粋なWeb標準技術のみ）
- **読み込み速度**: 初回表示 < 1秒

### コンテンツファースト
- **中身で勝負**: 技術的価値を明確に伝える構成
- **学術的信頼性**: 研究背景と理論的基盤を重視
- **実用性**: 具体的なコード例と使用方法

## 📁 ファイル構成

```
simple-website/
├── index.html          # メインページ（8KB）
├── styles.css          # スタイルシート（12KB）
├── script.js           # 最小限のJavaScript（2KB）
└── README.md           # ドキュメント
```

## 🚀 特徴

### パフォーマンス
- **ゼロ依存**: フレームワークやライブラリ不使用
- **最小JavaScript**: タブ切り替えとスムーススクロールのみ
- **軽量CSS**: Grid/Flexboxによる効率的レイアウト
- **HTTP/2対応**: 最適化されたリソース配信

### デザイン
- **アクセシビリティ**: WCAG準拠のコントラスト比
- **レスポンシブ**: モバイルファーストデザイン
- **タイポグラフィ**: 読みやすさを重視した文字設定
- **ダークテーマ対応**: システム設定に連動

### コンテンツ
- **Hero Section**: 言語の核心価値を明確に表現
- **Features**: 6つの主要機能を数学的記法で説明
- **Code Examples**: 3種類のタイプシステムデモ
- **Research Foundation**: 学術的背景と先行研究
- **Getting Started**: 実用的な導入ガイド

## 🛠️ ローカル開発

### HTTPサーバー起動
```bash
# Python 3
python3 -m http.server 8080

# Node.js (npx)
npx serve .

# PHP
php -S localhost:8080
```

### ブラウザでアクセス
```
http://localhost:8080
```

## 📊 パフォーマンス指標

### ファイルサイズ
- **index.html**: 8.2KB (gzip: 2.8KB)
- **styles.css**: 12.1KB (gzip: 3.2KB)  
- **script.js**: 1.8KB (gzip: 0.8KB)
- **合計**: 22.1KB (gzip: 6.8KB)

### 読み込み時間（推定）
- **高速回線**: < 0.5秒
- **標準回線**: < 1.0秒
- **低速回線**: < 2.0秒

### Lighthouse スコア（目標）
- **Performance**: 95+
- **Accessibility**: 100
- **Best Practices**: 100
- **SEO**: 100

## 🔧 カスタマイズ

### 色の変更
```css
:root {
  --primary-blue: #3b82f6;
  --text-dark: #1e293b;
  --text-light: #64748b;
  --background: #ffffff;
}
```

### フォントの変更
```css
body {
  font-family: 'Your-Font', -apple-system, BlinkMacSystemFont, sans-serif;
}

code {
  font-family: 'Your-Mono-Font', 'JetBrains Mono', monospace;
}
```

## 🌐 デプロイメント

### 静的ホスティング
- **GitHub Pages**: `gh-pages` ブランチに配置
- **Netlify**: ディレクトリをドラッグ&ドロップ
- **Vercel**: `vercel --prod` コマンド実行
- **CloudFlare Pages**: Git連携で自動デプロイ

### CDN最適化
```html
<!-- 圧縮とキャッシュ設定例 -->
<meta http-equiv="Cache-Control" content="public, max-age=31536000">
<link rel="preload" href="styles.css" as="style">
<link rel="preload" href="script.js" as="script">
```

## 🎨 デザインシステム

### タイポグラフィ階層
- **H1**: 3.5rem (56px) - ヒーロータイトル
- **H2**: 2.5rem (40px) - セクションタイトル
- **H3**: 1.5rem (24px) - サブセクション
- **Body**: 1rem (16px) - 本文テキスト
- **Code**: 0.875rem (14px) - コードブロック

### スペーシング
- **セクション間**: 6rem (96px)
- **要素間**: 1-2rem (16-32px)
- **コンテナ幅**: 1200px最大幅

### ブレークポイント
- **モバイル**: < 768px
- **タブレット**: 768px - 1024px
- **デスクトップ**: > 1024px

## 📝 コンテンツガイドライン

### トーン
- **学術的**: 理論的な厳密性を保持
- **実用的**: 具体的な使用例を提供
- **謙虚**: 先行研究への敬意を表明
- **革新的**: 独自の貢献を明確化

### SEO対策
- **メタタグ**: 適切なdescriptionとkeywords
- **構造化データ**: Schema.orgマークアップ
- **内部リンク**: 関連ページへの誘導
- **外部リンク**: 権威的なソースへの参照

## 🔒 セキュリティ

### ヘッダー設定
```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Referrer-Policy: origin-when-cross-origin
```

### HTTPS必須
- **SSL証明書**: Let's Encrypt推奨
- **HSTS**: Strict-Transport-Security有効化
- **Mixed Content**: HTTPリソース使用禁止

## 📈 アナリティクス

### 軽量トラッキング
```html
<!-- プライバシー重視の軽量アナリティクス -->
<script async src="https://analytics.example.com/script.js"></script>
```

### 測定指標
- **ページビュー**: 基本的な閲覧数
- **滞在時間**: コンテンツエンゲージメント
- **バウンス率**: ユーザー体験の質
- **リファラー**: トラフィックソース分析

---

**設計思想**: 「中身で勝負する」プロダクトにふさわしい、軽量で高品質なウェブサイト