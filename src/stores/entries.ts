import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { entryApi, type Entry, type EntryAdjustment, type CreateEntryParams, type EntryFilter } from "@/api/entries";

export const useEntryStore = defineStore("entries", () => {
  const entries = ref<Entry[]>([]);
  const adjustments = ref<Record<string, EntryAdjustment[]>>({});
  const currentBookId = ref<string | null>(null);
  const loading = ref(false);

  const assets = computed(() => entries.value.filter((e) => e.kind === "asset" && !e.closedAt));
  const liabilities = computed(() =>
    entries.value.filter((e) => e.kind === "liability" && !e.closedAt)
  );
  const totalAssets = computed(() => assets.value.reduce((s, e) => s + e.value, 0));
  const totalLiabilities = computed(() => liabilities.value.reduce((s, e) => s + e.value, 0));
  const netWorth = computed(() => totalAssets.value - totalLiabilities.value);

  async function fetchEntries(bookId: string, filter?: EntryFilter) {
    currentBookId.value = bookId;
    loading.value = true;
    try {
      entries.value = await entryApi.list(bookId, filter);
    } finally {
      loading.value = false;
    }
  }

  async function createEntry(params: CreateEntryParams) {
    const entry = await entryApi.create(params);
    entries.value.push(entry);
    return entry;
  }

  async function updateEntry(id: string, params: Partial<CreateEntryParams>) {
    const updated = await entryApi.update(id, params);
    const idx = entries.value.findIndex((e) => e.id === id);
    if (idx !== -1) entries.value[idx] = updated;
    return updated;
  }

  async function deleteEntry(id: string) {
    await entryApi.remove(id);
    entries.value = entries.value.filter((e) => e.id !== id);
  }

  async function adjustValue(id: string, newValue: number, reason?: string) {
    await entryApi.adjustValue(id, newValue, reason);
    const entry = entries.value.find((e) => e.id === id);
    if (entry) entry.value = newValue;
    // 清除缓存的调整历史，触发重新加载
    delete adjustments.value[id];
  }

  async function fetchAdjustments(entryId: string) {
    adjustments.value[entryId] = await entryApi.listAdjustments(entryId);
  }

  return {
    entries,
    adjustments,
    loading,
    assets,
    liabilities,
    totalAssets,
    totalLiabilities,
    netWorth,
    fetchEntries,
    createEntry,
    updateEntry,
    deleteEntry,
    adjustValue,
    fetchAdjustments,
  };
});
