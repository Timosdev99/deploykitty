use crate::profile::{AiAgent, Database, DeploymentTarget, ReverseProxy};

pub struct Pipeline;

impl Pipeline {
    pub fn script_for(target: &DeploymentTarget) -> &'static str {
        match target {
            DeploymentTarget::Hardening => {
                r#"set -e
echo "=== System Hardening ==="
ufw --force reset
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw --force enable
echo "UFW enabled with SSH only"

sed -i 's/^#PermitRootLogin yes/PermitRootLogin prohibit-password/' /etc/ssh/sshd_config
sed -i 's/^#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
systemctl restart sshd
echo "SSH hardened"

apt-get update -qq
apt-get install -y -qq fail2ban
systemctl enable --now fail2ban
echo "fail2ban installed and running"
echo "=== Hardening complete ==="
"#
            }
            DeploymentTarget::Database(Database::Postgres) => {
                r#"set -e
echo "=== Installing Postgres ==="
apt-get update -qq
apt-get install -y -qq postgresql postgresql-contrib
systemctl enable --now postgresql
echo "Postgres $(psql --version) installed"
echo "=== Postgres done ==="
"#
            }
            DeploymentTarget::Database(Database::Redis) => {
                r#"set -e
echo "=== Installing Redis ==="
apt-get update -qq
apt-get install -y -qq redis-server
systemctl enable --now redis-server
echo "Redis $(redis-server --version) installed"
echo "=== Redis done ==="
"#
            }
            DeploymentTarget::Database(Database::MongoDB) => {
                r#"set -e
echo "=== Installing MongoDB ==="
apt-get update -qq
apt-get install -y -qq gnupg curl
curl -fsSL https://www.mongodb.org/static/pgp/server-7.0.asc | gpg --dearmor -o /usr/share/keyrings/mongodb-server-7.0.gpg
echo "deb [ signed-by=/usr/share/keyrings/mongodb-server-7.0.gpg ] http://repo.mongodb.org/apt/debian bookworm/mongodb-org/7.0 main" | tee /etc/apt/sources.list.d/mongodb-org-7.0.list
apt-get update -qq
apt-get install -y -qq mongodb-org
systemctl enable --now mongod
echo "MongoDB installed"
echo "=== MongoDB done ==="
"#
            }
            DeploymentTarget::ReverseProxy(ReverseProxy::Caddy) => {
                r#"set -e
echo "=== Installing Caddy ==="
apt-get update -qq
apt-get install -y -qq debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list
apt-get update -qq
apt-get install -y -qq caddy
echo "Caddy $(caddy version) installed"
echo "=== Caddy done ==="
"#
            }
            DeploymentTarget::ReverseProxy(ReverseProxy::Nginx) => {
                r#"set -e
echo "=== Installing Nginx ==="
apt-get update -qq
apt-get install -y -qq nginx
systemctl enable --now nginx
echo "Nginx $(nginx -v 2>&1) installed"
echo "=== Nginx done ==="
"#
            }
            DeploymentTarget::AiAgent(AiAgent::Hermes) => {
                r#"set -e
echo "=== Installing Hermes AI Agent ==="
apt-get update -qq
apt-get install -y -qq python3 python3-pip git
git clone https://github.com/NousResearch/hermes.git /opt/hermes 2>/dev/null || (cd /opt/hermes && git pull)
cd /opt/hermes
pip3 install -r requirements.txt 2>/dev/null || true
echo "Hermes agent installed at /opt/hermes"
echo "=== Hermes done ==="
"#
            }
            DeploymentTarget::AiAgent(AiAgent::OpenClaw) => {
                r#"set -e
echo "=== Installing OpenClaw AI Agent ==="
apt-get update -qq
apt-get install -y -qq python3 python3-pip git
git clone https://github.com/openclaw/openclaw.git /opt/openclaw 2>/dev/null || (cd /opt/openclaw && git pull)
cd /opt/openclaw
pip3 install -r requirements.txt 2>/dev/null || true
echo "OpenClaw agent installed at /opt/openclaw"
echo "=== OpenClaw done ==="
"#
            }
            DeploymentTarget::DockerCompose => {
                r#"set -e
echo "=== Installing Docker ==="
apt-get update -qq
apt-get install -y -qq ca-certificates curl
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
chmod a+r /etc/apt/keyrings/docker.asc
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
apt-get update -qq
apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-compose-plugin
systemctl enable --now docker
echo "Docker $(docker --version) installed"
echo "=== Docker done ==="
"#
            }
            DeploymentTarget::Binary => {
                r#"set -e
echo "=== Binary Deployment ==="
echo "Upload your binary to /opt/app and run it with systemd"
echo "Example:"
echo "  scp ./my-app user@host:/opt/app/"
echo "  create /etc/systemd/system/my-app.service"
echo "  systemctl enable --now my-app"
echo "=== Binary deploy scaffold done ==="
"#
            }
        }
    }

    pub fn target_label(target: &DeploymentTarget) -> &'static str {
        match target {
            DeploymentTarget::Hardening => "System Hardening",
            DeploymentTarget::Database(Database::Postgres) => "Postgres",
            DeploymentTarget::Database(Database::Redis) => "Redis",
            DeploymentTarget::Database(Database::MongoDB) => "MongoDB",
            DeploymentTarget::ReverseProxy(ReverseProxy::Caddy) => "Caddy",
            DeploymentTarget::ReverseProxy(ReverseProxy::Nginx) => "Nginx",
            DeploymentTarget::AiAgent(AiAgent::Hermes) => "Hermes Agent",
            DeploymentTarget::AiAgent(AiAgent::OpenClaw) => "OpenClaw Agent",
            DeploymentTarget::DockerCompose => "Docker Compose",
            DeploymentTarget::Binary => "Binary Deployment",
        }
    }
}
