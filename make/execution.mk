# Admit Make itself before recipe failures can be ignored or execution skipped.
# Shared companions: scripts/ci/check-make-execution.sh
ifneq ($(origin _shared_make_execution_checked),override)
override _shared_make_execution_checked := yes
# $(shell) does not reliably export this invocation's computed Make flags.
# Pass them explicitly, shell-quoted, to the existing behavioral probe. Its
# isolated Makefile never loads consumer recipes. Keep real caller flags intact.
ifneq ($(shell MAKEFLAGS='$(subst ','"'"',$(MAKEFLAGS))' MFLAGS='$(subst ','"'"',$(MFLAGS))' MAKEOVERRIDES= GNUMAKEFLAGS= bash '$(subst ','"'"',$(SHARED_TOOLING_ROOT))/scripts/ci/check-make-execution.sh' '$(subst ','"'"',$(MAKE))' >/dev/null 2>&1 && printf admitted),admitted)
$(error Shared tooling requires Make recipe execution and failure propagation; remove ignore-errors, dry-run, touch and question modes)
endif
endif
