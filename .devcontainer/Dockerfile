FROM hashicorp/terraform:latest

ENV AWS_CLI_VERSION=2.4.14
RUN apk add \
        curl \
        cmake \
        gcc \
        gcompat \
        libffi-dev \
        make \
        musl-dev \
        openssl-dev \
        py3-pip \
        py3-setuptools \
        python3 \
        python3-dev \
        sudo \
        zip \
    && pip3 install wheel \
    && curl -s "https://codeload.github.com/aws/aws-cli/zip/refs/tags/${AWS_CLI_VERSION}" -o awscliv2.zip \
    && unzip awscliv2.zip \
    && cd "aws-cli-${AWS_CLI_VERSION}" \
    && python3 setup.py install

RUN apk add \
        gcc \
        linux-headers \
        musl-dev \
        py3-pip \
        python3-dev \
    && pip3 install aws2-wrap \
    && mkdir -p /home/exec/.aws/sso/cache \
    && echo $'[default]\n\
sso_start_url = https://itmettkede.awsapps.com/start#/\n\
sso_region = eu-central-1\n\
sso_account_id = 518105872728\n\
sso_role_name = ReadOnlyAccessExtended\n\
region = eu-central-1\n\
output = json' > /home/exec/.aws/config

RUN apk update \
    && apk upgrade
RUN apk add \
        bash \
        curl \
        jq \
        nano \
        openssl \
        sudo \
    && addgroup -S exec \
    && adduser -S -D  \
        -h /home/exec \
        -g exec \
        -s /bin/bash \
        -g exec \
        exec \
    && adduser exec wheel \
    && sed -e 's;^# \(%wheel.*NOPASSWD.*\);\1;g' -i /etc/sudoers \
    && echo $'PROMPT_COMMAND="history -a"\n' > /home/exec/.bashrc \
    && chown exec:exec /home/exec -R
    
RUN apk add \
        gnupg \
    && echo $'export GPG_TTY="$(tty)"\n\
gpg-connect-agent updatestartuptty /bye >/dev/null 2>/dev/null\n\
export SSH_AUTH_SOCK=$(gpgconf --list-dirs agent-ssh-socket)\n\
gpgconf --launch gpg-agent >/dev/null 2>/dev/null' >> /home/exec/.bashrc

USER exec

ENV PATH="${PATH}:/home/exec/.cargo/bin"

RUN sudo apk add \
        curl \
        libgcc \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && cargo install --locked cargo-outdated \
    && cargo install --locked cargo-readme \
    && cargo install --locked cargo-release \
    && cargo install --locked cargo-update

RUN sudo apk add \
        openssl \
        openssl-dev \
        perl
