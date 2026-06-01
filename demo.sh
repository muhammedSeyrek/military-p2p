#!/usr/bin/env bash
#
# Military P2P — Demo orchestration script.
#
# Usage:
# ./demo.sh setup → Reset databases, load CSV, initialize commanders
# ./demo.sh start → Start 7 commander servers
# ./demo.sh stop → Stop servers
# ./demo.sh status → Server status
# ./demo.sh dispatch → Distribute test message
# ./demo.sh read NAME → Read message with commander NAME (mehmet, ali, ...)
# ./demo.sh read-all → Read from all 7 commanders in order
# ./demo.sh tamper → Tampering test: corrupt a part, read, see error
# ./demo.sh reset → Reset all operations (but keep init)
# ./demo.sh full-demo → Complete end-to-end demo (excluding setup)

set -euo pipefail

# === Configuration ===

GENERAL_DB="postgresql://mp:mp_dev_pass@pg-general/general"
KEYS_DIR="/tmp/keys"
LOGS_DIR="/tmp/logs"

# Commander definitions: slug → (email, port)
declare -A EMAILS=(
  [mehmet]="mehmetyilmaz@karakuvvetleri.mil.tr"
  [ali]="aliaslan@karakuvvetleri.mil.tr"
  [zeynep]="zeynepkaradag@karakuvvetleri.mil.tr"
  [koray]="korayaydin@karakuvvetleri.mil.tr"
  [aylin]="aylinkaya@karakuvvetleri.mil.tr"
  [emre]="emredemir@karakuvvetleri.mil.tr"
  [burak]="burakarslan@karakuvvetleri.mil.tr"
)

declare -A PORTS=(
  [mehmet]=8443
  [ali]=8444
  [zeynep]=8445
  [koray]=8446
  [aylin]=8447
  [emre]=8448
  [burak]=8449
)

declare -A NAMES=(
  [mehmet]="Mehmet Yilmaz (Orgeneral)"
  [ali]="Ali Aslan (Korgeneral)"
  [zeynep]="Zeynep Karadag (Tümgeneral)"
  [koray]="Koray Aydin (Tümgeneral)"
  [aylin]="Aylin Kaya (Tuggeneral)"
  [emre]="Emre Demir (Tuggeneral)"
  [burak]="Burak Arslan (Albay)"
)

SLUGS=(mehmet ali zeynep koray aylin emre burak)

# Color printout
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# === Helpers ===

info()    { echo -e "${BLUE}[INFO]${NC}  $*"; }
ok()      { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $*"; }
err()     { echo -e "${RED}[ERR]${NC}   $*"; }

ensure_binary() {
  if [[ ! -x ./target/release/mp-node-general ]] || [[ ! -x ./target/release/mp-node-commander ]]; then
    info "Release binary missing, building..."
    cargo build --release -p mp-node-general -p mp-node-commander
    ok "Build completed."
  fi
}

drop_general_tables() {
  PGPASSWORD=mp_dev_pass psql -h pg-general -U mp -d general -q \
    -c "DROP TABLE IF EXISTS operation_recipients, operations, commanders, units, _sqlx_migrations CASCADE;" >/dev/null 2>&1 || true
}

drop_commander_tables() {
  local slug=$1
  PGPASSWORD=mp_dev_pass psql -h pg-$slug -U mp -d $slug -q \
    -c "DROP TABLE IF EXISTS parts, operations, peer_directory, self_info, _sqlx_migrations CASCADE;" >/dev/null 2>&1 || true
}

truncate_commander_runtime_tables() {
  local slug=$1
  PGPASSWORD=mp_dev_pass psql -h pg-$slug -U mp -d $slug -q \
    -c "TRUNCATE parts, operations CASCADE;" >/dev/null 2>&1 || true
}

# === Commands ===

cmd_setup() {
  ensure_binary

  info "1/6 Cleaning all DBs..."
  drop_general_tables
  for slug in "${SLUGS[@]}"; do
    drop_commander_tables "$slug"
  done
  ok "DBs cleaned."

  info "2/6 Loading 7 commanders from CSV to General HQ DB..."
  rm -rf "$KEYS_DIR"
  ./target/release/mp-node-general --db-url "$GENERAL_DB" load-csv \
    --file data/commanders.csv \
    --keys-dir "$KEYS_DIR" >/dev/null
  ok "7 commanders + RSA keys generated."

  info "3/6 Changing network addresses to localhost..."
  for slug in "${SLUGS[@]}"; do
    local port=${PORTS[$slug]}
    local email=${EMAILS[$slug]}
    PGPASSWORD=mp_dev_pass psql -h pg-general -U mp -d general -q \
      -c "UPDATE commanders SET network_address = 'http://localhost:$port' WHERE email = '$email';" >/dev/null
  done
  ok "Addresses updated."

  info "4/6 Initializing each commander node..."
  for slug in "${SLUGS[@]}"; do
    local email=${EMAILS[$slug]}
    ./target/release/mp-node-commander \
      --db-url "postgresql://mp:mp_dev_pass@pg-$slug/$slug" \
      init --email "$email" \
           --private-key-file "$KEYS_DIR/key-$slug.pem" >/dev/null 2>&1
    echo "    ✓ $slug initialized"
  done

  info "5/6 Updating addresses in peer directories..."
  for db in "${SLUGS[@]}"; do
    for target_slug in "${SLUGS[@]}"; do
      local target_port=${PORTS[$target_slug]}
      local target_email=${EMAILS[$target_slug]}
      PGPASSWORD=mp_dev_pass psql -h pg-$db -U mp -d $db -q \
        -c "UPDATE peer_directory SET network_address = 'http://localhost:$target_port' WHERE email = '$target_email';" >/dev/null
    done
  done
  ok "Peer directories updated."

  info "6/6 Setup completed."
  echo ""
  ok "System ready. To start servers: ./demo.sh start"
}

cmd_start() {
  ensure_binary
  mkdir -p "$LOGS_DIR"

  info "Closing old servers..."
  pkill -f "mp-node-commander.*serve" 2>/dev/null || true
  sleep 1

  info "Starting 7 commander servers..."
  for slug in "${SLUGS[@]}"; do
    local port=${PORTS[$slug]}
    RUST_LOG=info nohup ./target/release/mp-node-commander \
      --db-url "postgresql://mp:mp_dev_pass@pg-$slug/$slug" \
      serve --port "$port" > "$LOGS_DIR/$slug.log" 2>&1 &
    echo "    ✓ $slug → port $port (PID $!)"
  done

  info "Waiting for 3 seconds..."
  sleep 3
  cmd_status
}

cmd_stop() {
  info "Closing all servers..."
  pkill -f "mp-node-commander.*serve" 2>/dev/null || true
  sleep 1
  ok "Servers stopped."
}

cmd_status() {
  echo ""
  echo "═══════════════════════════════════════════════"
  echo " Server status"
  echo "═══════════════════════════════════════════════"
  for slug in "${SLUGS[@]}"; do
    local port=${PORTS[$slug]}
    local result
    result=$(curl -s -m 2 "http://localhost:$port/health" 2>/dev/null || echo "DOWN")
    if [[ "$result" == "ok" ]]; then
      echo -e "  ${GREEN}●${NC} $slug ($port): ${NAMES[$slug]}"
    else
      echo -e "  ${RED}●${NC} $slug ($port): DOWN"
    fi
  done
  echo ""
}

cmd_dispatch() {
  ensure_binary

  info "Distributing 7 different military orders..."
  echo ""

  ./target/release/mp-node-general --db-url "$GENERAL_DB" dispatch \
    --name "Operation Sand" \
    --to "mehmetyilmaz@karakuvvetleri.mil.tr:Deploy to the eastern border" \
    --to "aliaslan@karakuvvetleri.mil.tr:Provide food supply for 25000 personnel" \
    --to "zeynepkaradag@karakuvvetleri.mil.tr:Close the airspace" \
    --to "korayaydin@karakuvvetleri.mil.tr:Reinforce border outposts" \
    --to "aylinkaya@karakuvvetleri.mil.tr:Secure the communication center" \
    --to "emredemir@karakuvvetleri.mil.tr:Establish a defense line on the western flank" \
    --to "burakarslan@karakuvvetleri.mil.tr:Artillery unit stand by for fire"
}

cmd_read() {
  ensure_binary
  local slug=${1:-aylin}

  if [[ -z "${EMAILS[$slug]:-}" ]]; then
    err "Unknown commander: $slug"
    echo "Valid options: ${SLUGS[*]}"
    exit 1
  fi

  ./target/release/mp-node-commander \
    --db-url "postgresql://mp:mp_dev_pass@pg-$slug/$slug" \
    read --operation "Operation Sand"
}

cmd_read_all() {
  ensure_binary

  for slug in "${SLUGS[@]}"; do
    echo ""
    echo "═══════════════════════════════════════════════"
    echo " $slug is reading..."
    echo "═══════════════════════════════════════════════"
    ./target/release/mp-node-commander \
      --db-url "postgresql://mp:mp_dev_pass@pg-$slug/$slug" \
      read --operation "Operation Sand" 2>&1 | grep -A 10 "Operation:" || warn "$slug could not read"
  done
}

cmd_tamper() {
  ensure_binary

  warn "Starting TAMPERING TEST..."
  echo ""
  info "1) Corrupting ciphertext of part_index=3 in Koray's DB."
  PGPASSWORD=mp_dev_pass psql -h pg-koray -U mp -d koray -q \
    -c "UPDATE parts SET ciphertext_chunk = decode('deadbeefcafebabe1234567890abcdef0011223344556677889900aabbccddeeff', 'hex') WHERE part_index = 3;" >/dev/null
  ok "Part corrupted (attacker controls a malicious commander)."

  echo ""
  info "2) Attempting to read from Aylin. The system should detect it via Merkle tree."
  echo ""

  set +e
  ./target/release/mp-node-commander \
    --db-url postgresql://mp:mp_dev_pass@pg-aylin/aylin \
    read --operation "Operation Sand" 2>&1
  local exit_code=$?
  set -e

  echo ""
  if [[ $exit_code -ne 0 ]]; then
    ok "TAMPERING detected! Operation deleted."
  else
    err "Tampering could not be detected (bug!)"
  fi

  echo ""
  info "3) Operation status in Aylin's DB:"
  PGPASSWORD=mp_dev_pass psql -h pg-aylin -U mp -d aylin \
    -c "SELECT name FROM operations;"
}

cmd_reset() {
  info "Deleting all operations (commander records preserved)..."
  PGPASSWORD=mp_dev_pass psql -h pg-general -U mp -d general -q \
    -c "TRUNCATE operation_recipients, operations CASCADE;" >/dev/null

  for slug in "${SLUGS[@]}"; do
    truncate_commander_runtime_tables "$slug"
  done
  ok "Operations cleared. System ready for dispatch."
}

cmd_full_demo() {
  echo ""
  echo "════════════════════════════════════════════════════════"
  echo "  MILITARY P2P — FULL DEMO"
  echo "════════════════════════════════════════════════════════"
  echo ""

  info "Checking system status..."
  cmd_status

  echo ""
  read -p "Press Enter to continue..."

  cmd_reset

  echo ""
  info "Demo 1: General HQ distributes 7 different military orders..."
  echo ""
  read -p "Press Enter to continue..."
  cmd_dispatch

  echo ""
  info "Demo 2: All commanders read their own messages..."
  echo ""
  read -p "Press Enter to continue..."
  cmd_read_all

  echo ""
  info "Demo 3: Tampering test (Detection via Merkle + self-healing)..."
  echo ""
  read -p "Press Enter to continue..."
  cmd_tamper

  echo ""
  info "Demo 4: System reset, returning to normal state..."
  echo ""
  read -p "Press Enter to continue..."
  cmd_reset
  cmd_dispatch

  echo ""
  ok "Demo completed!"
}

# === Main ===

usage() {
  cat <<EOF
Military P2P Demo Script

Usage: $0 <command> [arg]

Commands:
  setup              Reset DB + Load CSV + init 7 commanders (1 time)
  start              Start 7 servers in background
  stop               Stop servers
  status             Health status of servers
  dispatch           Distribute operation to 7 commanders with distinct messages
  read <slug>        Read message with a commander (mehmet/ali/zeynep/koray/aylin/emre/burak)
  read-all           Read from all 7 commanders sequentially
  tamper             Tampering test: corrupt part, read, see error
  reset              Delete operations (commander records remain)
  full-demo          Complete end-to-end demo (interactive)

Example workflow:
  ./demo.sh setup         # first time
  ./demo.sh start
  ./demo.sh dispatch
  ./demo.sh read aylin
  ./demo.sh tamper
EOF
}

case "${1:-}" in
  setup)     cmd_setup ;;
  start)     cmd_start ;;
  stop)      cmd_stop ;;
  status)    cmd_status ;;
  dispatch)  cmd_dispatch ;;
  read)      shift; cmd_read "$@" ;;
  read-all)  cmd_read_all ;;
  tamper)    cmd_tamper ;;
  reset)     cmd_reset ;;
  full-demo) cmd_full_demo ;;
  ""|-h|--help) usage ;;
  *)         err "Unknown command: $1"; usage; exit 1 ;;
esac