#!/usr/bin/env bash
# ----------------------------------------------------------------
# Synopsis.
# ----------------------------------------------------------------

# 1. Start v1 running at current mainnet commit.
# 2. Waits for genesis era to complete.
# 4. Delegates from an unused account.
# 5. Waits for the auction delay to take effect.
# 7. Asserts delegation is in auction info.
# 8. Stages the network for upgrade.
# 9. Assert v2 nodes run & the chain advances (new blocks are generated).
# 10. Waits 1 era.
# 13. Waits for the auction delay to take effect.
# 15. Asserts delegatee is NO LONGER in auction info.
# 16. Run Health Checks
# 17. Successful test cleanup.

# ----------------------------------------------------------------
# Imports.
# ----------------------------------------------------------------

source "$NCTL/sh/utils/main.sh"
source "$NCTL/sh/node/svc_$NCTL_DAEMON_TYPE.sh"
source "$NCTL/sh/assets/upgrade.sh"
source "$NCTL/sh/scenarios/common/itst.sh"

# ----------------------------------------------------------------
# MAIN
# ----------------------------------------------------------------

# Main entry point.
function _main()
{
    local STAGE_ID=${1}

    if [ ! -d "$(get_path_to_stage "$STAGE_ID")" ]; then
        log "ERROR :: stage $STAGE_ID has not been built - cannot run scenario"
        exit 1
    fi

    _step_01 "$STAGE_ID"
    _step_02

    # Set initial protocol version for use later.
    INITIAL_PROTOCOL_VERSION=$(get_node_protocol_version 1)
    _step_03
    _step_04
    _step_05
    _step_06
    _step_07
    _step_08
    _step_09
    exit 1
    _step_10
    _step_11
    _step_12
    _step_13
    _step_14
    _step_15
    _step_16
    _step_17
}

# Step 01: Start network from pre-built stage.
function _step_01()
{
    local STAGE_ID=${1}
    local PATH_TO_STAGE
    local PATH_TO_PROTO1

    PATH_TO_STAGE=$(get_path_to_stage "$STAGE_ID")
    pushd "$PATH_TO_STAGE"
    PATH_TO_PROTO1=$(ls -d */ | sort | head -n 1 | tr -d '/')
    popd

    log_step_upgrades 1 "starting network from stage ($STAGE_ID)"

    source "$NCTL/sh/assets/setup_from_stage.sh" \
            stage="$STAGE_ID" \
            accounts_path="$NCTL/overrides/upgrade_scenario_3.pre.accounts.toml"
    source "$NCTL/sh/node/start.sh" node=all
}

# Step 02: Await era-id >= 1.
function _step_02()
{
    log_step_upgrades 2 "awaiting genesis era completion"

    do_await_genesis_era_to_complete 'false'
}

# Step 03: Delegate from a nodes account
function _step_03()
{
    local NODE_ID=${1:-'5'}
    local ACCOUNT_ID=${2:-'7'}
    local AMOUNT=${3:-'500000000000'}

    log_step_upgrades 3 "Delegating $AMOUNT from account-$ACCOUNT_ID to validator-$NODE_ID"

    source "$NCTL/sh/contracts-auction/do_delegate.sh" \
            amount="$AMOUNT" \
            delegator="$ACCOUNT_ID" \
            validator="$NODE_ID"
}

# Step 04: Await 1 era
function _step_04()
{
    log_step_upgrades 4 "Awaiting Auction_Delay = 1 + 1"
    nctl-await-n-eras offset='2' sleep_interval='5.0' timeout='300'
}

# Step 05: Assert USER_ID is a delegatee
function _step_05()
{
    local USER_ID=${1:-'7'}
    local USER_PATH
    local HEX
    local AUCTION_INFO_FOR_HEX
    local TIMEOUT_SEC

    TIMEOUT_SEC='0'

    USER_PATH=$(get_path_to_user "$USER_ID")
    HEX=$(cat "$USER_PATH"/public_key_hex | tr '[:upper:]' '[:lower:]')

    log_step_upgrades 5 "Asserting user-$USER_ID is a delegatee"

    while [ "$TIMEOUT_SEC" -le "60" ]; do
        AUCTION_INFO_FOR_HEX=$(nctl-view-chain-auction-info | jq --arg node_hex "$HEX" '.auction_state.bids[]| select(.bid.delegators[].public_key | ascii_downcase == $node_hex)')
        if [ ! -z "$AUCTION_INFO_FOR_HEX" ]; then
            log "... user-$USER_ID found in auction info delegators!"
            log "... public_key_hex: $HEX"
            echo "$AUCTION_INFO_FOR_HEX"
            break
        else
            TIMEOUT_SEC=$((TIMEOUT_SEC + 1))
            log "... timeout=$TIMEOUT_SEC: delegatee not yet detected"
            sleep 1
            if [ "$TIMEOUT_SEC" = '60' ]; then
                log "ERROR: Could not find $HEX in auction info delegators!"
                echo "$(nctl-view-chain-auction-info)"
                exit 1
            fi
        fi
    done
}

# Step 06: Undelegate previous user
function _step_06()
{
    local NODE_ID=${1:-'5'}
    local ACCOUNT_ID=${2:-'7'}
    local AMOUNT=${3:-'500000000000'}

    log_step_upgrades 6 "Undelegating $AMOUNT to account-$ACCOUNT_ID from validator-$NODE_ID"

    source "$NCTL/sh/contracts-auction/do_delegate_withdraw.sh" \
            amount="$AMOUNT" \
            delegator="$ACCOUNT_ID" \
            validator="$NODE_ID"
}

# Emergency Restart this bitch with a validator swap
function _step_07()
{
    local ACTIVATE_ERA
    local ERA_ID
    local SWITCH_BLOCK
    local STATE_HASH
    local TRUSTED_HASH
    local PROTOCOL_VERSION

    ACTIVATE_ERA="$(get_chain_era)"
    ERA_ID=$((ACTIVATE_ERA - 1))
    SWITCH_BLOCK=$(get_switch_block "1" "32" "" "$ERA_ID")
    STATE_HASH=$(echo "$SWITCH_BLOCK" | jq -r '.header.state_root_hash')
    TRUSTED_HASH=$(echo "$SWITCH_BLOCK" | jq -r '.hash')
    PROTOCOL_VERSION='2_0_0'

    log_step_upgrades 7 "Emergency restart with validator swap"
    log "...emergency upgrade activation era = $ACTIVATE_ERA"
    log "...state hash = $STATE_HASH"
    log "...trusted hash = $TRUSTED_HASH"
    log "...new protocol version = $PROTOCOL_VERSION"

    do_node_stop_all

    for NODE_ID in $(seq 1 "$(get_count_of_nodes)"); do
        log "...preparing $NODE_ID"
        _emergency_upgrade_node "$PROTOCOL_VERSION" "$ACTIVATE_ERA" "$NODE_ID" "$STATE_HASH" 1 "$(get_count_of_genesis_nodes)" "$NCTL_CASPER_HOME/resources/local/config.toml" "$NCTL_CASPER_HOME/resources/local/chainspec.toml.in"
        log "...starting $NODE_ID"
        #HACK FOR NOW
        #sed -i 's/\[block_proposer\]//g' $(get_path_to_node_config $NODE_ID)/"$PROTOCOL_VERSION"/config.toml
        #sed -i 's/max_execution_delay.*//g' $(get_path_to_node_config $NODE_ID)/"$PROTOCOL_VERSION"/config.toml
        #sed -i 's/max_execution_delay.*//g' $(get_path_to_node_config $NODE_ID)/"$PROTOCOL_VERSION"/config.toml
        do_node_start "$NODE_ID" "$TRUSTED_HASH"
    done
    sleep 10
}

function _step_08()
{
    log_step_upgrades 8 "Awaiting Auction_Delay = 1 + 1"
    nctl-await-n-eras offset='2' sleep_interval='5.0' timeout='300'
}

# Step 08: Assert USER_ID is NOT a delegatee
function _step_09()
{
    local USER_ID=${1:-'7'}
    local USER_PATH
    local HEX
    local AUCTION_INFO_FOR_HEX

    USER_PATH=$(get_path_to_user "$USER_ID")
    HEX=$(cat "$USER_PATH"/public_key_hex | tr '[:upper:]' '[:lower:]')
    AUCTION_INFO_FOR_HEX=$(nctl-view-chain-auction-info | jq --arg node_hex "$HEX" '.auction_state.bids[]| select(.bid.delegators[].public_key | ascii_downcase == $node_hex)')

    log_step_upgrades 9 "Asserting user-$USER_ID is NOT a delegatee"

    if [ ! -z "$AUCTION_INFO_FOR_HEX" ]; then
        log "ERROR: user-$USER_ID found in auction info delegators!"
        log "... public_key_hex: $HEX"
        echo "$AUCTION_INFO_FOR_HEX"
        exit 1
    else
        log "... Could not find $HEX in auction info delegators! [expected]"
    fi
}

# Step 10: Run NCTL health checks
function _step_10()
{
    # restarts=6 - Nodes that upgrade
    log_step_upgrades 10 "running health checks"
    source "$NCTL"/sh/scenarios/common/health_checks.sh \
            errors='0' \
            equivocators='0' \
            doppels='0' \
            crashes=0 \
            restarts=6 \
            ejections=0
}

# Step 11: Terminate.
function _step_11()
{
    log_step_upgrades 11 "test successful - tidying up"

    source "$NCTL/sh/assets/teardown.sh"

    log_break
}

# ----------------------------------------------------------------
# ENTRY POINT
# ----------------------------------------------------------------

unset _STAGE_ID
unset INITIAL_PROTOCOL_VERSION

for ARGUMENT in "$@"
do
    KEY=$(echo "$ARGUMENT" | cut -f1 -d=)
    VALUE=$(echo "$ARGUMENT" | cut -f2 -d=)
    case "$KEY" in
        stage) _STAGE_ID=${VALUE} ;;
        *)
    esac
done

_main "${_STAGE_ID:-1}"
