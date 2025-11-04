# omd マニュアル

omd CLI ツールの完全なリファレンスガイド。

## 目次

1. [はじめに](#はじめに)
2. [インストール](#インストール)
3. [クイックスタート](#クイックスタート)
4. [設定](#設定)
5. [コマンドリファレンス](#コマンドリファレンス)
6. [プロジェクト設定](#プロジェクト設定)
7. [高度な使い方](#高度な使い方)
8. [トラブルシューティング](#トラブルシューティング)
9. [ベストプラクティス](#ベストプラクティス)

## はじめに

omd（oh-my-dockers）は、Docker 開発環境を管理するための包括的な CLI ツールです。以下の機能を提供します：

- **ローカルプロジェクト設定**: 各プロジェクトが独自の `omd.toml` 設定ファイルを管理
- **自動ネットワーク管理**: Docker ネットワークを自動的に作成・管理
- **ポート競合検出**: 複数のプロジェクト間でのポート競合を防止
- **リバースプロキシ統合**: HTTPS アクセスのための Caddy 設定を自動生成
- **スマートコンテナ検出**: `docker-compose.yml` からコンテナ情報を自動解析
- **集中レジストリ**: すべてのプロジェクトとポート割り当てを追跡

### 主要な概念

**プロジェクトベースのワークフロー**: 集中管理型の設定システムとは異なり、omd はローカルプロジェクトディレクトリで動作します。各プロジェクトには独自の `omd.toml` 設定ファイルがあります。

**ポートレジストリ**: omd はグローバルレジストリ（`~/.oh-my-dockers/registry.json`）を維持し、どのポートがどのプロジェクトで使用されているかを追跡し、競合を防ぎます。

**Caddy 統合**: リバースプロキシ設定を自動生成し、カスタムドメインで HTTPS 経由でサービスにアクセスできるようにします。

## インストール

### システム要件

- **オペレーティングシステム**: macOS、Linux、または Windows（WSL2）
- **Docker**: バージョン 20.10 以降
- **Docker Compose**: バージョン 2.0 以降
- **Rust**: 1.70 以降（ソースからビルドする場合）
- **Caddy**: 最新版（リバースプロキシ機能用）

### ソースからのビルド

```bash
# リポジトリをクローン
git clone <repository-url>
cd oh-my-dockers

# リリースバイナリをビルド
cargo build --release

# バイナリは target/release/omd にあります
```

### システム PATH へのインストール

```bash
# グローバルにインストール
cargo install --path .

# インストールを確認
omd --help
```

## クイックスタート

### 基本的なワークフロー

```bash
# 1. プロジェクトに移動
cd /path/to/your/project

# 2. omd 設定を初期化
omd init

# 3. docker-compose.yml が存在することを確認
# （既存の docker-compose.yml ファイル）

# 4. インフラストラクチャを設定（ネットワーク、Caddy など）
omd up

# 5. コンテナを起動
docker compose up -d

# 6. サービスにアクセス
# https://your-project.local
```

### 完全な例

```bash
# 新しいプロジェクトを作成
mkdir my-api
cd my-api

# omd を初期化
omd init
# Project name [my-api]: 
# Domain [my-api.local]: 
# Network name [my-api-net]: 
# Do you want to configure Caddy routes now? [y/N]: n

# docker-compose.yml を作成
cat > docker-compose.yml <<EOF
services:
  postgres:
    image: postgres:15
    container_name: my-api-postgres
    ports:
      - "5432:5432"
    environment:
      POSTGRES_PASSWORD: secret
    networks:
      - my-api-net

  api:
    build: .
    container_name: my-api-backend
    ports:
      - "3000:3000"
    depends_on:
      - postgres
    networks:
      - my-api-net

networks:
  my-api-net:
EOF

# インフラストラクチャを設定
omd up
# ℹ docker-compose.yml を解析中...
# ℹ ホストポートを検出: 5432, 3000
# ✓ ポート競合なし
# ✓ ネットワークを作成
# ✓ Caddy 設定を生成
# ✓ プロジェクト my-api を設定完了！

# サービスを起動
docker compose up -d

# https://api.my-api.local でアクセス可能
```

## 設定

### 設定ディレクトリ

ツールはデフォルトの設定ディレクトリとして `~/.oh-my-dockers` を使用します。

**ディレクトリ構造：**

```
~/.oh-my-dockers/
├── config.toml          # グローバル設定
├── registry.json        # ポート割り当て付きプロジェクトレジストリ
└── caddy/
    ├── Caddyfile        # メイン Caddy 設定
    ├── certs/           # SSL 証明書
    └── projects/        # 生成されたプロジェクト別 Caddy 設定
```

**カスタム設定ディレクトリ：**

```bash
export OH_MY_DOCKERS_DIR="/custom/path"
```

### グローバル設定

グローバル設定ファイルは `~/.oh-my-dockers/config.toml` にあります：

```toml
[global]
# Caddy ネットワーク名
caddy_network = "caddy-net"

# ディレクトリ（設定ディレクトリからの相対パス）
caddy_projects_dir = "caddy/projects"
caddy_certs_dir = "caddy/certs"

[defaults]
# デフォルトタイムゾーン
timezone = "Asia/Tokyo"

# ネットワーク定義
[networks]
# Caddy リバースプロキシネットワーク
caddy-net = {}
```

## コマンドリファレンス

### omd init

現在のディレクトリに `omd.toml` 設定を初期化します。

```bash
omd init
```

**インタラクティブなプロンプト：**
- プロジェクト名（デフォルト：現在のディレクトリ名）
- ドメイン（デフォルト：`{プロジェクト名}.local`）
- ネットワーク名（デフォルト：`{プロジェクト名}-net`）
- Caddy ルートの設定（オプション）

**出力**: 現在のディレクトリに `omd.toml` を作成します。

### omd project up

プロジェクトのインフラストラクチャを設定します（プロジェクトディレクトリから実行）。

```bash
cd /path/to/project
omd up
```

**実行内容：**
1. 現在のディレクトリから `omd.toml` を読み取り
2. `docker-compose.yml` を解析してポートとコンテナ名を抽出
3. 他の登録済みプロジェクトとのポート競合をチェック
4. 存在しない場合は Docker ネットワークを作成
5. **Caddy が起動していない場合は自動的に起動**
6. Caddy リバースプロキシ設定を生成
7. グローバルレジストリにプロジェクトを登録

**重要**: これはプロジェクトのコンテナを起動しません。`docker compose up -d` を別途実行してください。

### omd project down

プロジェクト設定を削除します（プロジェクトディレクトリから実行）。

```bash
cd /path/to/project
omd down
```

**実行内容：**
1. 現在のディレクトリから `omd.toml` を読み取り
2. Caddy 設定を削除
3. グローバルレジストリからプロジェクトを登録解除
4. Caddy をリロード

**重要**: これはコンテナを停止しません。`docker compose down` を別途実行してください。

### omd project list

登録済みのすべてのプロジェクトをリストします。

```bash
omd project list
```

**出力例：**

```
登録済みプロジェクト:

  • my-api
    パス: /Users/dev/projects/my-api
    ドメイン: my-api.local
    ネットワーク: my-api-net
    ポート: 5432, 3000

  • my-web
    パス: /Users/dev/projects/my-web
    ドメイン: my-web.local
    ネットワーク: my-web-net
    ポート: 8080, 8443
```

### omd network list

すべての Docker ネットワークをリストします。

```bash
omd network list
```

### omd proxy add

リバースプロキシルールを手動で追加します。

```bash
omd proxy add DOMAIN TARGET
```

**例：**

```bash
omd proxy add example.local backend:3000
```

### omd proxy remove

リバースプロキシルールを削除します。

```bash
omd proxy remove DOMAIN
```

### omd proxy list

すべてのプロキシルールをリストします。

```bash
omd proxy list
```

### omd proxy reload

Caddy 設定をリロードします。

```bash
omd proxy reload
```

### omd ports

すべてのネットワーク全体のポートマッピングを表示します。

```bash
# すべてのネットワーク
omd ports

# 特定のネットワーク
omd ports show my-api-net
```

## プロジェクト設定

### omd.toml の構造

`omd.toml` ファイルはプロジェクトディレクトリにあります：

```toml
[project]
# プロジェクト名（コンテナ命名に使用）
name = "my-project"

# このプロジェクトのドメイン
domain = "my-project.local"

# オプション：docker-compose ファイルのパス（プロジェクトディレクトリからの相対パス）
# 指定されていない場合のデフォルトは "docker-compose.yml"
# compose_file = "docker/docker-compose.yml"
# compose_file = "docker-compose.dev.yml"

[network]
# このプロジェクトの Docker ネットワーク名
name = "my-project-net"

[caddy]
# カスタム Caddy ルート（オプション）
routes = {}
```

### 自動ルート生成

`[caddy.routes]` が空または指定されていない場合、omd は `docker-compose.yml` からルートを自動生成します：

**docker-compose.yml の例：**

```yaml
services:
  frontend:
    image: my-frontend
    ports:
      - "8080:80"
    networks:
      - myapp-net

  backend:
    image: my-backend
    ports:
      - "3000:3000"
    networks:
      - myapp-net
```

**生成されるルート：**

- `frontend.my-project.local` → `frontend-container:80`
- `backend.my-project.local` → `backend-container:3000`

### カスタムルート

カスタムルートで自動ルーティングを上書きできます：

```toml
[project]
name = "my-project"
domain = "my-project.local"

[network]
name = "my-project-net"

[caddy.routes]
# カスタムルート: サブドメイン -> コンテナ:ポート
api = "my-backend-container:3000"
app = "my-frontend-container:80"
admin = "my-admin-panel:8080"
```

**生成されるルート：**

- `api.my-project.local` → `my-backend-container:3000`
- `app.my-project.local` → `my-frontend-container:80`
- `admin.my-project.local` → `my-admin-panel:8080`

## 高度な使い方

### ポート競合検出

`omd up` を実行すると、ツールはグローバルレジストリでポート競合をチェックします：

**シナリオ：**

プロジェクト A が PostgreSQL のポート 5432 を使用しています。

プロジェクト B を設定しようとしますが、これもポート 5432 を使用しています。

**結果：**

```
✗ ポート競合を検出:
  ポート 5432 は既にプロジェクト project-a で使用されています

ポート競合のため続行できません。
docker-compose.yml を更新して別のポートを使用してください。
```

**解決方法：**

プロジェクト B の `docker-compose.yml` を更新：

```yaml
services:
  postgres:
    image: postgres:15
    ports:
      - "5433:5432"  # 5432:5432 から変更
```

### 複数プロジェクト

ポート競合がない限り、複数のプロジェクトを同時に実行できます：

```bash
# プロジェクト 1
cd ~/projects/api-service
omd init
omd up
docker compose up -d
# https://api-service.local でアクセス

# プロジェクト 2
cd ~/projects/web-app
omd init
omd up
docker compose up -d
# https://web-app.local でアクセス
```

## トラブルシューティング

### ポートがすでに使用中

**エラー：**

```
✗ ポート競合を検出:
  ポート 5432 は既にプロジェクト another-project で使用されています
```

**解決方法：**

`docker-compose.yml` のホストポートを変更：

```yaml
ports:
  - "5433:5432"  # 別のホストポートを使用
```

### omd.toml が見つからない

**エラー：**

```
現在のディレクトリに omd.toml が見つかりません。'omd init' を実行して作成してください。
```

**解決方法：**

プロジェクトディレクトリで `omd init` を実行するか、正しいディレクトリに移動してください。

### docker-compose.yml が見つからない

**エラー：**

```
現在のディレクトリに docker-compose.yml が見つかりません。
'omd up' を実行する前に docker-compose.yml を作成してください。
```

**解決方法：**

プロジェクトディレクトリに `docker-compose.yml` ファイルを作成してください。

## ベストプラクティス

### 1. プロジェクトごとに 1 つの omd.toml

`omd.toml` をプロジェクトのルートディレクトリ（`docker-compose.yml` と同じ場所）に保管してください。

### 2. 明示的なコンテナ名を使用

本番環境のような環境では、コンテナ名を明示的に指定してください：

```yaml
services:
  api:
    image: my-api
    container_name: my-project-api
```

### 3. ポート割り当てを文書化

`docker-compose.yml` にコメントを追加：

```yaml
services:
  postgres:
    ports:
      - "5432:5432"  # 標準 PostgreSQL ポート
```

### 4. 一貫した命名

プロジェクト名、ネットワーク名、ドメイン全体で一貫した命名を使用：

```toml
[project]
name = "my-awesome-app"
domain = "my-awesome-app.local"

[network]
name = "my-awesome-app-net"
```

### 5. 起動前に確認

`docker compose up -d` の前に必ず `omd up` を実行：

```bash
omd up              # インフラストラクチャを設定
docker compose up -d     # コンテナを起動
```

### 6. 削除後のクリーンアップ

プロジェクトを削除する場合は、まず `omd down` を実行：

```bash
cd /path/to/project
docker compose down  # コンテナを停止
omd down             # 設定を削除
cd ..
rm -rf project       # ディレクトリを削除
```

---

詳細については、[GitHub リポジトリ](https://github.com/your-repo/oh-my-dockers) をご覧ください。
