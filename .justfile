set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# ===============
# ===Variables===

COCOS_BIN := if os() == "macos" { '' } else { '' }
PY_SHEBANG := if os() == "windows" { "python" } else { "/usr/bin/env python" }

# ===Variables===
# ===============

[private]
default:
    just --list

__chk_env:
    just __chk_seaorm_bin

__chk_seaorm_bin:
    #!{{ PY_SHEBANG }}
    import os
    to_install = ["sea-orm-cli"]
    cmd_str = "cargo install --list"
    r = os.popen(cmd_str)
    for line in r.readlines():
        for bin in to_install:
            # print(line, bin, line.startswith(bin), "-->> line bin")
            if not line.startswith(" ") and line.startswith(bin):
                to_install.remove(bin)
                print("%s env ready!"%bin)
    if len(to_install) > 0:
        for bin in to_install:
            os.system("cargo install {}".format(bin))




# 构建当天镜像
__docker_build image_tag:
    docker build -t {{ image_tag }} -f ./Dockerfile  .
    docker tag {{ image_tag }} registry.cn-beijing.aliyuncs.com/life-jt/{{ image_tag }}

# 构建单一服务
__cargo_build build_type svr="default":
    [ "{{ build_type }}" = "release" ] && cargo build --release || cargo build
    mkdir -p {{ justfile_directory() }}/out

# 准备image out文件夹
__image_output:
    cp {{ justfile_directory() }}/rsvr_$(date "+%Y-%m-%d").tar.gz   {{ justfile_directory() }}/out/rsvr_$(date "+%Y-%m-%d").tar.gz
    cp {{ justfile_directory() }}/dpm_docker.yml {{ justfile_directory() }}/out/dpm.yml

# 准备binary out文件夹
__binary_output:
    cp {{ justfile_directory() }}/target/release/hs_frog   {{ justfile_directory() }}/out/hs_frog
    cp {{ justfile_directory() }}/dpm.yml {{ justfile_directory() }}/out/dpm.yml

# 发布docker image 到aliyun acr
__pub_aliyun_acr image_tag usr="picboo" pwd="9e4F&aMN*=0>":
    docker login --username=dwbpicboo registry.cn-beijing.aliyuncs.com --password='9e4F&aMN*=0>'
    docker push registry.cn-beijing.aliyuncs.com/life-jt/{{ image_tag }}

# 在本地docker内运行hs_frog
run_hub_svr:
    docker stop hs_frog_latest
    docker rm hs_frog_latest
    docker rmi -f dwbmio/hs_frog
    docker pull dwbmio/hs_frog:latest
    docker run  --name hs_frog_latest -p 8001:8001 -d  dwbmio/hs_frog:latest

# 构建docker并发布镜像到阿里云(`$image_tag` EX: hs_frog:latest)
build_to_hub image_tag:
    just __docker_build {{ image_tag }}
    just __pub_aliyun_acr {{ image_tag }}

# 同步数据库的entity到本地
sync_entity db="hs_frog" path="entity/src":
    just __chk_env
    DATABASE_URL=$(grep DB_MAIN_ADDR .env | cut -d '=' -f2) && \
    echo $DATABASE_URL && \
     sea-orm-cli generate entity -l -o {{ path }} \
    --database-url $DATABASE_URL \
    --with-serde both \
    --serde-skip-deserializing-primary-key \
    --date-time-crate chrono \
    --serde-skip-hidden-column \
    --serde-skip-option-none


# graphql 同步
sync_graphql path="./entity_graphql/src":
    just __chk_env
    DB_GRAPHQL=$(grep DB_MAIN_ADDR .env | cut -d '=' -f2) && \
    echo $DB_GRAPHQL && \
     sea-orm-cli generate entity -l -o {{ path }}  \
    --database-url $DB_GRAPHQL \
    --with-serde both \
    --serde-skip-deserializing-primary-key \
    --date-time-crate chrono \
    --serde-skip-hidden-column \
    --serde-skip-option-none \
    --seaography


graphql_gen path="./graphql/entity/src":
    just __chk_env
    DATABASE_URL=$(grep DB_MAIN_ADDR .env | cut -d '=' -f2) && \
    seaography-cli ./ graphql/entity/src $DATABASE_URL seaography-mysql-example
