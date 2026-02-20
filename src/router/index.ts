import { createRouter, createWebHashHistory } from "vue-router";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      redirect: "/books",
    },
    {
      path: "/",
      component: () => import("@/layouts/AppLayout.vue"),
      children: [
        {
          path: "books",
          name: "books",
          component: () => import("@/pages/books/BooksPage.vue"),
        },
        {
          path: "books/:id",
          name: "book-detail",
          component: () => import("@/pages/books/BookDetailPage.vue"),
        },
        {
          path: "books/:id/records",
          name: "book-records",
          component: () => import("@/pages/records/RecordsPage.vue"),
        },
        {
          path: "views/:id",
          name: "view-stats",
          component: () => import("@/pages/views/ViewStatsPage.vue"),
        },
        {
          path: "books/:id/snapshots",
          name: "book-snapshots",
          component: () => import("@/pages/snapshots/SnapshotsPage.vue"),
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
