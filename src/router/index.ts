import { createRouter, createWebHashHistory } from "vue-router";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      component: () => import("@/layouts/AppLayout.vue"),
      children: [
        {
          path: "",
          name: "home",
          redirect: "/records",
        },
        {
          path: "records",
          name: "records",
          component: () => import("@/pages/records/RecordsPage.vue"),
        },
        {
          path: "assets",
          name: "assets",
          component: () => import("@/pages/books/BookDetailPage.vue"),
        },
        {
          path: "stats",
          name: "stats",
          component: () => import("@/pages/views/ViewStatsPage.vue"),
        },
        // 更多 / 设置
        {
          path: "more",
          name: "more",
          component: () => import("@/pages/more/MorePage.vue"),
        },
        // 账本管理（从更多页进入）
        {
          path: "books",
          name: "books",
          component: () => import("@/pages/books/BooksPage.vue"),
        },
        {
          path: "settings/categories",
          name: "settings-categories",
          component: () => import("@/pages/settings/CategoriesPage.vue"),
        },
        {
          path: "settings/tags",
          name: "settings-tags",
          component: () => import("@/pages/settings/TagsPage.vue"),
        },
      ],
    },
  ],
});

export default router;
