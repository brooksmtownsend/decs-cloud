# Copyright 2015-2018 Capital One Services, LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

COLOR ?= always # Valid COLOR options: {always, auto, never}
CARGO = cargo --color $(COLOR)

.PHONY: all bench build check clean doc test update

all: build

bench:
	@$(CARGO) bench

build:
	@$(CARGO) build
	wascap sign target/wasm32-unknown-unknown/debug/game_loop.wasm target/wasm32-unknown-unknown/debug/game_loop_s.wasm -a ./.keys/account.nk -m ./.keys/module.nk -s -g -k -c decs:timer
	wascap sign target/wasm32-unknown-unknown/debug/system_mgr.wasm target/wasm32-unknown-unknown/debug/system_mgr_s.wasm -a ./.keys/account.nk -m ./.keys/module.nk -s -g -k -c decs:timer
	wascap sign target/wasm32-unknown-unknown/debug/shard_mgr.wasm target/wasm32-unknown-unknown/debug/shard_mgr_s.wasm -a ./.keys/account.nk -m ./.keys/module.nk -s -g -k
	wascap sign target/wasm32-unknown-unknown/debug/component_mgr.wasm target/wasm32-unknown-unknown/debug/component_mgr_s.wasm -a ./.keys/account.nk -m ./.keys/module.nk -s -g -k

check:
	@$(CARGO) check

clean:
	@$(CARGO) clean

doc:
	@$(CARGO) doc

test: build
	@$(CARGO) test

update:
	@$(CARGO) update

release:	
	wascap sign target/wasm32-unknown-unknown/release/game_loop.wasm target/wasm32-unknown-unknown/release/game_loop_s.wasm -a ./.keys/account.nk -m ./.keys/module.nk -s -g -k -c decs:timer
	wascap sign target/wasm32-unknown-unknown/release/system_mgr.wasm target/wasm32-unknown-unknown/release/system_mgr_s.wasm -a ./.keys/account.nk -m ./.keys/module.nk -s -g -k -c decs:timer
	wascap sign target/wasm32-unknown-unknown/release/shard_mgr.wasm target/wasm32-unknown-unknown/release/shard_mgr_s.wasm -a ./.keys/account.nk -m ./.keys/module.nk -s -g -k
	wascap sign target/wasm32-unknown-unknown/release/component_mgr.wasm target/wasm32-unknown-unknown/release/component_mgr_s.wasm -a ./.keys/account.nk -m ./.keys/module.nk -s -g -k

docker: release
	docker build -t decscloud/game_loop -f ./Dockerfile.gameloop .
	docker build -t decscloud/system_mgr -f ./Dockerfile.systemmgr .
	docker build -t decscloud/shard_mgr -f ./Dockerfile.shardmgr .
	docker build -t decscloud/component_mgr -f ./Dockerfile.componentmgr .
